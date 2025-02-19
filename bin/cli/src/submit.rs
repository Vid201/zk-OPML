use alloy::{
    eips::BlockNumberOrTag,
    network::EthereumWallet,
    primitives::{Address, Bytes, U256},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::Filter,
    signers::local::LocalSigner,
    sol,
    sol_types::SolEvent,
};
use candle_core::Tensor;
use candle_onnx::eval::simple_eval_one;
use futures_util::stream::StreamExt;
use rand::Rng;
use sha2::Digest;
use std::{collections::HashMap, str::FromStr};
use tracing::info;
use zkopml_ml::{data::tensor_hash, onnx::load_onnx_model};

#[derive(clap::Args, Debug, Clone)]
pub struct SubmitArgs {
    #[arg(long, short, help = "Verbosity level (0-4)", action = clap::ArgAction::Count)]
    pub v: u8,

    /// Address of the Ethereum node endpoint to use
    #[clap(long)]
    pub eth_node_address: String,

    /// Address of the ModelRegistry contract
    #[clap(long)]
    pub model_registry_address: Address,

    /// Address of the FaultProof contract
    #[clap(long)]
    pub fault_proof_address: Address,

    /// Secret key to use for requesting the inference
    #[clap(long)]
    pub user_key: String,

    /// Model id to use
    #[clap(long)]
    pub model_id: u8,

    /// Path to the model file (ONNX)
    #[clap(long)]
    pub model_path: String,

    /// Input shape of the model
    #[clap(long, value_delimiter = ',')]
    pub input_shape: Vec<usize>,

    /// Output shape of the model
    #[clap(long, value_delimiter = ',')]
    pub output_shape: Vec<usize>,

    /// Whether to submit wrong result (for testing purposes)
    #[clap(long, short)]
    pub defect: bool,
}

sol!(
    #[derive(Debug)]
    event InferenceRequested(uint256 modelId, uint256 inferenceId, address requester, bytes inputData, bytes32 inputDataHash);
    #[derive(Debug)]
    event ChallengeCreated(uint256 challengeId, uint256 inferenceId, address responder, address challenger);
    #[derive(Debug)]
    event OperatorExecutionProposed(
        uint256 challengeId, uint256 operatorPosition, bytes32 inputDataHash, bytes32 outputDataHash
    );
    #[derive(Debug)]
    event OperatorExecutionResponded(uint256 challengeId, uint256 operatorPosition, bool input, bool output);
    #[derive(Debug)]
    event ChallengeResolved(uint256 challengeId, bool success, address winner);
);

pub async fn submit(args: SubmitArgs) -> anyhow::Result<()> {
    // Initialize the user wallet
    let user_signer = LocalSigner::from_str(&args.user_key)?;
    let user_wallet = EthereumWallet::from(user_signer);
    let ws_connect = WsConnect::new(args.eth_node_address);
    let user_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&user_wallet)
        .on_ws(ws_connect)
        .await?;
    info!("User address: {}", user_wallet.default_signer().address());

    // Listen for inference requests
    let _model_registry =
        zkopml_contracts::ModelRegistry::new(args.model_registry_address, user_provider.clone());
    let inference_request_filter = Filter::new()
        .address(args.model_registry_address)
        .event("InferenceRequested(uint256,uint256,address,bytes,bytes32)")
        .from_block(BlockNumberOrTag::Latest);
    let sub = user_provider
        .subscribe_logs(&inference_request_filter)
        .await?;
    let mut stream = sub.into_stream();

    let input_shape = args.input_shape.clone();

    while let Some(log) = stream.next().await {
        // Parse the data
        info!("Received inference request: {:?}", log);
        let request = InferenceRequested::decode_log_data(log.data(), false);
        if request.is_err() {
            info!("Failed to decode the request data");
            continue;
        }
        let request = request.unwrap();
        info!("Decoded request: {:?}", request);
        let model_id: U256 = request.modelId;
        let inference_id: U256 = request.inferenceId;
        let input_data_arr_u8 = request.inputData.as_ref();
        let num_f32s = input_data_arr_u8.len() / 4;
        let input_data = vec![unsafe {
            let f32_slice =
                std::slice::from_raw_parts(input_data_arr_u8.as_ptr() as *const f32, num_f32s);
            f32_slice.to_vec()
        }];
        info!(
            "Model id: {}, Inference id: {}, Input data: {:?}",
            model_id, inference_id, input_data
        );

        // Perform the inference
        // TODO: read the model file from IPFS based on model id
        // For now, we are going to assume there is only one model
        info!("Reading the model file from {}", args.model_path);
        let model_path = args.model_path.clone();
        let model = load_onnx_model(&model_path)?;

        let input_data: Vec<f32> = input_data.into_iter().flat_map(|v| v).collect();
        let mut inputs: HashMap<String, Tensor> = HashMap::new();
        model.prepare_inputs(&mut inputs, input_data, input_shape.clone())?;

        let mut inference_data: HashMap<U256, Vec<([u8; 32], [u8; 32])>> = HashMap::new();

        // If the defect flag is set, randomly select an operator to produce a defect
        let defect_index = if args.defect {
            let mut rng = rand::rng();
            let random_index = rng.random_range(0..model.num_operators());
            Some(random_index)
        } else {
            None
        };

        for i in 0..model.num_operators() {
            let node = model.get_node(i).unwrap();

            // Calculate hash of the input data
            let mut input_hashes = HashMap::new();

            for (name, tensor) in inputs.iter() {
                let hash = tensor_hash(&tensor);
                input_hashes.insert(name.clone(), hash);
            }
            let mut input_entries = input_hashes.iter().collect::<Vec<_>>();
            input_entries.sort_by(|a, b| a.0.cmp(b.0));
            let mut hasher = sha2::Sha256::new();
            hasher.update(serde_json::to_string(&input_entries).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
            let input_hash: [u8; 32] = hasher.finalize().into();

            simple_eval_one(&node, &mut inputs)?;

            let mut defect_produced = false;

            // Calculate hash of the output data
            let mut input_hashes = HashMap::new();
            for (name, tensor) in inputs.iter_mut() {
                if let Some(defect_index) = defect_index {
                    if !defect_produced && i == defect_index {
                        if node.output.contains(name) {
                            info!(
                                "Augmenting the output data {} with a defect, index: {}",
                                name, defect_index
                            );
                            let tensor_pow = tensor.powf(2.0f64)?;
                            *tensor = tensor_pow;
                            defect_produced = true;
                        }
                    }
                }
                let hash = tensor_hash(tensor);
                input_hashes.insert(name.clone(), hash);
            }
            let mut input_entries = input_hashes.iter().collect::<Vec<_>>();
            input_entries.sort_by(|a, b| a.0.cmp(b.0));
            let mut hasher = sha2::Sha256::new();
            hasher.update(serde_json::to_string(&input_entries).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
            let output_hash: [u8; 32] = hasher.finalize().into();

            inference_data
                .entry(inference_id)
                .or_insert_with(Vec::new)
                .push((input_hash, output_hash));
        }

        let result = inputs.clone();

        info!("Inference result: {:?}", result["output"].to_string());
        let mut input_hashes = HashMap::new();
        for (name, tensor) in inputs.iter() {
            let hash = tensor_hash(tensor);
            input_hashes.insert(name.clone(), hash);
        }
        let mut input_entries = input_hashes.iter().collect::<Vec<_>>();
        input_entries.sort_by(|a, b| a.0.cmp(b.0));
        let mut hasher = sha2::Sha256::new();
        hasher.update(serde_json::to_string(&input_entries).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
        let hash: [u8; 32] = hasher.finalize().into();

        // Submit the result
        let output_data: Vec<f32> = result["output"].flatten_all()?.to_vec1::<f32>()?;
        let output_data = Bytes::from_iter(unsafe {
            std::slice::from_raw_parts(output_data.as_ptr() as *const u8, output_data.len() * 4)
                .iter()
        });
        let model_registry = zkopml_contracts::ModelRegistry::new(
            args.model_registry_address,
            user_provider.clone(),
        );
        let tx = model_registry
            .respondInference(inference_id, output_data, hash.into())
            .send()
            .await?;
        info!("Transaction hash: {}", tx.tx_hash());
        info!("Inference {} responded", inference_id);

        std::thread::sleep(std::time::Duration::from_secs(10));

        // Listen for challenge requests
        let fault_proof =
            zkopml_contracts::FaultProof::new(args.fault_proof_address, user_provider.clone());
        let challenge_request_filter = Filter::new()
            .address(args.fault_proof_address)
            .from_block(BlockNumberOrTag::Latest);
        let sub = user_provider
            .subscribe_logs(&challenge_request_filter)
            .await?;
        let mut stream = sub.into_stream();

        let mut challenge_ids: Vec<U256> = Vec::new();

        while let Some(log) = stream.next().await {
            // Parse the data
            // TODO: support multiple challenge IDs
            info!("Received challenge event: {:?}", log);
            match log.topic0() {
                Some(&ChallengeCreated::SIGNATURE_HASH) => {
                    let request = ChallengeCreated::decode_log_data(log.data(), false);
                    if request.is_err() {
                        info!("Failed to decode the request data");
                        continue;
                    }
                    let request = request.unwrap();
                    info!(
                        "Challenge id {} for inference id {} created",
                        request.challengeId, request.inferenceId
                    );
                    if inference_id == request.inferenceId {
                        challenge_ids.push(request.challengeId);
                    }
                }
                Some(&OperatorExecutionProposed::SIGNATURE_HASH) => {
                    let request = OperatorExecutionProposed::decode_log_data(log.data(), false);
                    if request.is_err() {
                        info!("Failed to decode the request data");
                        continue;
                    }
                    let request = request.unwrap();
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    if challenge_ids.contains(&request.challengeId) {
                        let operator_position = request.operatorPosition;
                        info!(
                            "Operator execution proposed for challenge id {} at position {}",
                            request.challengeId, request.operatorPosition
                        );
                        let (input_data_hash, output_data_hash) = inference_data
                            .get(&inference_id)
                            .unwrap()[operator_position.to::<usize>()];
                        let input_data_match = input_data_hash == request.inputDataHash;
                        let output_data_match = output_data_hash == request.outputDataHash;
                        info!(
                            "Operator execution response for challenge id {} at position {}: {}, {}",
                            request.challengeId, request.operatorPosition, input_data_match, output_data_match
                        );
                        let tx = fault_proof
                            .respondOperatorExecution(
                                request.challengeId,
                                input_data_match,
                                output_data_match,
                            )
                            .send()
                            .await?;
                        info!("Transaction hash: {}", tx.tx_hash());
                        std::thread::sleep(std::time::Duration::from_secs(2));
                    }
                }
                Some(&ChallengeResolved::SIGNATURE_HASH) => {
                    let request = ChallengeResolved::decode_log_data(log.data(), false);
                    if request.is_err() {
                        info!("Failed to decode the request data");
                        continue;
                    }
                    let request = request.unwrap();
                    info!(
                        "Challenge id {} resolved, challenge actor winner: {}, address winner: {}",
                        request.challengeId, request.success, request.winner
                    );
                }
                _ => {
                    // info!("Unknown event signature: {:?}", log);
                }
            }

            // TODO: handle expired challenges
        }
    }

    Ok(())
}
