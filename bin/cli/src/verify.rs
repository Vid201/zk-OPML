use alloy::{
    eips::BlockNumberOrTag,
    hex::ToHexExt,
    network::EthereumWallet,
    primitives::{Address, Bytes, U256},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::Filter,
    signers::local::LocalSigner,
    sol,
    sol_types::SolEvent,
};
use candle_core::{DType, Tensor};
use candle_onnx::eval::{get_tensor, simple_eval_one};
use futures_util::StreamExt;
use sha2::Digest;
use sp1_sdk::{Prover, ProverClient, SP1Stdin, include_elf, network::FulfillmentStrategy};
use std::{collections::HashMap, str::FromStr};
use tracing::info;
use zkopml_ml::{data::tensor_hash, merkle::ModelMerkleTree, onnx::load_onnx_model};

#[derive(clap::Args, Debug, Clone)]
pub struct VerifyArgs {
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
}

sol!(
    #[derive(Debug)]
    event InferenceResponded(
        uint256 modelId, uint256 inferenceId, address responder, bytes outputData, bytes32 outputDataHash
    );
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

const ELF: &[u8] = include_elf!("zkopml-zk");

pub async fn verify(args: VerifyArgs) -> anyhow::Result<()> {
    // Initialize the user wallet
    let user_signer = LocalSigner::from_str(&args.user_key)?;
    let user_wallet = EthereumWallet::from(user_signer);
    let ws_connect = WsConnect::new(args.eth_node_address);
    let user_provider = ProviderBuilder::new()
        .wallet(&user_wallet)
        .connect_ws(ws_connect)
        .await?;
    info!("User address: {}", user_wallet.default_signer().address());

    // Listen for inference responses
    let _model_registry =
        zkopml_contracts::ModelRegistry::new(args.model_registry_address, user_provider.clone());
    let inference_response_filter = Filter::new()
        .address(args.model_registry_address)
        .event("InferenceResponded(uint256,uint256,address,bytes,bytes32)")
        .from_block(BlockNumberOrTag::Latest);
    let sub = user_provider
        .subscribe_logs(&inference_response_filter)
        .await?;
    let mut stream = sub.into_stream();

    while let Some(log) = stream.next().await {
        // Parse the data
        info!("Received inference response: {:?}", log);
        let response = InferenceResponded::decode_log_data(log.data());
        if response.is_err() {
            info!("Failed to decode the response data");
            continue;
        }
        let response = response.unwrap();
        info!("Decoded response: {:?}", response);
        let model_id: U256 = response.modelId;
        let inference_id: U256 = response.inferenceId;
        info!(
            "Model id: {}, Inference id: {}, Output data: {:?}",
            model_id, inference_id, response.outputData
        );

        // Get the inference input data
        let model_registry = zkopml_contracts::ModelRegistry::new(
            args.model_registry_address,
            user_provider.clone(),
        );
        let inference = model_registry.getInference(inference_id).call().await?;
        let input_data = inference.inputData;
        info!("Inference input data: {:?}", input_data);

        // Perform the inference
        // TODO: read the model file from IPFS based on model id
        // For now, we are going to assume there is only one model
        info!("Reading the model file from {}", args.model_path);
        let model_path = args.model_path.clone();
        let model = load_onnx_model(&model_path)?;

        let mut inputs: HashMap<String, Tensor> = serde_json::from_slice(&input_data).unwrap();

        for (name, tensor) in &inputs {
            match tensor.dtype() {
                DType::F32 => {
                    info!(
                        "Input tensor '{}': {:?}",
                        name,
                        tensor.flatten_all()?.to_vec1::<f32>()?
                    );
                }
                DType::F64 => {
                    info!(
                        "Input tensor '{}': {:?}",
                        name,
                        tensor.flatten_all()?.to_vec1::<f64>()?
                    );
                }
                _ => {}
            }
        }

        let mut inference_data: HashMap<U256, Vec<HashMap<String, Tensor>>> = HashMap::new();
        let mut inference_data_hashes: HashMap<U256, Vec<HashMap<String, [u8; 32]>>> =
            HashMap::new();
        let mut inference_hashes: HashMap<U256, Vec<([u8; 32], [u8; 32])>> = HashMap::new();

        for i in 0..model.num_operators() {
            if i == 0 {
                for t in model.graph().clone().unwrap().initializer.iter() {
                    let tensor = get_tensor(t, t.name.as_str())?;
                    inputs.insert(t.name.to_string(), tensor);
                }
            }

            inference_data
                .entry(inference_id)
                .or_insert_with(Vec::new)
                .push(inputs.clone());

            // Calculate hash of the input data
            let mut input_hashes = HashMap::new();
            for (name, tensor) in inputs.iter() {
                let hash = tensor_hash(tensor);
                input_hashes.insert(name.clone(), hash);
            }

            inference_data_hashes
                .entry(inference_id)
                .or_insert_with(Vec::new)
                .push(input_hashes.clone());

            let mut input_entries = input_hashes.iter().collect::<Vec<_>>();
            input_entries.sort_by(|a, b| a.0.cmp(b.0));
            let mut hasher = sha2::Sha256::new();
            hasher.update(serde_json::to_string(&input_entries).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
            let input_hash: [u8; 32] = hasher.finalize().into();

            let node = model.get_node(i).unwrap();
            simple_eval_one(&node, &mut inputs)?;

            // Calculate hash of the output data
            let mut input_hashes = HashMap::new();
            for (name, tensor) in inputs.iter() {
                let hash = tensor_hash(tensor);
                input_hashes.insert(name.clone(), hash);
            }

            let mut input_entries = input_hashes.iter().collect::<Vec<_>>();
            input_entries.sort_by(|a, b| a.0.cmp(b.0));
            let mut hasher = sha2::Sha256::new();
            hasher.update(serde_json::to_string(&input_entries).unwrap().as_bytes()); // TODO: figure out how to more efficiently hash a tensor
            let output_hash: [u8; 32] = hasher.finalize().into();

            inference_hashes
                .entry(inference_id)
                .or_insert_with(Vec::new)
                .push((input_hash, output_hash));
        }

        let mut result = HashMap::new();

        for output in model.graph().unwrap().output.iter() {
            if let Some(tensor) = inputs.get(output.name.as_str()) {
                info!(
                    "Inference result for {}: {:?}",
                    output.name,
                    tensor.to_string()
                );
                result.insert(output.name.clone(), tensor.clone());
            }
        }

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

        // Compare the result with the expected output
        let output_data =
            Bytes::copy_from_slice(serde_json::to_string(&result).unwrap().as_bytes());
        if output_data == response.outputData && hash == response.outputDataHash {
            info!("Output data matches the expected result, not challenging");
            continue;
        }

        info!(
            "Output data does not match the expected result, challenging inference {}",
            inference_id
        );
        std::thread::sleep(std::time::Duration::from_secs(10));
        let fault_proof =
            zkopml_contracts::FaultProof::new(args.fault_proof_address, user_provider.clone());

        let mut challenges: Vec<U256> = Vec::new();

        // Create challenge
        let tx = fault_proof.createChallenge(inference_id).send().await?;
        info!("Transaction hash: {}", tx.tx_hash());
        std::thread::sleep(std::time::Duration::from_secs(10));
        let challenge_id = fault_proof.challengeCounter().call().await? - U256::from(1);
        info!("Challenge created with id: {}", challenge_id);

        challenges.push(challenge_id);

        // Propose first operator execution
        let mut low = 0;
        let mut high = model.num_operators() - 1;
        let mut mid = (low + high) / 2;
        let (input_data_hash, output_data_hash) = inference_hashes.get(&inference_id).unwrap()[mid];
        let tx = fault_proof
            .proposeOperatorExecution(
                challenge_id,
                input_data_hash.into(),
                output_data_hash.into(),
            )
            .send()
            .await?;
        info!("Transaction hash: {}", tx.tx_hash());
        info!(
            "Operator execution for operator {} proposed with input data hash: {:?}, output data hash: {:?}",
            mid,
            input_data_hash.encode_hex(),
            output_data_hash.encode_hex()
        );

        // Listen for challenge requests
        let challenge_request_filter = Filter::new()
            .address(args.fault_proof_address)
            .from_block(BlockNumberOrTag::Latest);
        let sub = user_provider
            .subscribe_logs(&challenge_request_filter)
            .await?;
        let mut stream = sub.into_stream();

        while let Some(log) = stream.next().await {
            // Parse the data
            // TODO: support multiple challenge IDs
            info!("Received challenge event: {:?}", log);
            match log.topic0() {
                Some(&OperatorExecutionResponded::SIGNATURE_HASH) => {
                    let response = OperatorExecutionResponded::decode_log_data(log.data());
                    if response.is_err() {
                        info!("Failed to decode the response data");
                        continue;
                    }
                    let response = response.unwrap();
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    if challenges.contains(&response.challengeId) {
                        let input_data_match = response.input;
                        let output_data_match = response.output;
                        info!(
                            "Operator execution response: input data match: {}, output data match: {}",
                            input_data_match, output_data_match
                        );
                        match (input_data_match, output_data_match) {
                            (true, true) => {
                                // Move right
                                low = mid + 1;
                                mid = (low + high) / 2;
                                let (input_data_hash, output_data_hash) =
                                    inference_hashes.get(&inference_id).unwrap()[mid];
                                info!(
                                    "Operator execution for operator {} proposed with input data hash: {:?}, output data hash: {:?}",
                                    mid,
                                    input_data_hash.encode_hex(),
                                    output_data_hash.encode_hex()
                                );
                                let tx = fault_proof
                                    .proposeOperatorExecution(
                                        challenge_id,
                                        input_data_hash.into(),
                                        output_data_hash.into(),
                                    )
                                    .send()
                                    .await?;
                                info!("Transaction hash: {}", tx.tx_hash());
                                std::thread::sleep(std::time::Duration::from_secs(2));
                            }
                            (true, false) => {
                                // Do the SP1 zkVM verification
                                let mut stdin = SP1Stdin::new();

                                // Write the merkle tree root hash
                                let merkle_tree = ModelMerkleTree::new(
                                    model.graph().unwrap().node,
                                    model.graph().unwrap(),
                                );
                                stdin.write(&merkle_tree.root());

                                // Write the index of the operator
                                let leaf_indices = vec![mid];
                                stdin.write(&leaf_indices);

                                // Write the hashes of the leaves
                                let leaf_hashes = merkle_tree.leaves_hashes(leaf_indices.clone());
                                stdin.write(&leaf_hashes);

                                // Write the total number of leaves
                                let total_leaves = merkle_tree.total_leaves();
                                stdin.write(&total_leaves);

                                // Write the merkle proof
                                let merkle_proof: Vec<u8> =
                                    merkle_tree.prove(leaf_indices).to_bytes();
                                stdin.write(&merkle_proof);

                                // Write inputs
                                let node = model.get_node(mid).unwrap();
                                let mut inputs =
                                    inference_data.get(&inference_id).unwrap()[mid].clone();
                                inputs.retain(|k: &String, _| node.input.contains(k));
                                let mut inputs_raw: HashMap<String, Tensor> = HashMap::new();
                                for (name, tensor) in inputs.iter() {
                                    let mut init = false;
                                    for t in model.graph().unwrap().initializer.iter() {
                                        if name == &t.name {
                                            let mut input_name = name.clone();
                                            input_name.push_str("graph_initializer");
                                            inputs_raw.insert(input_name, tensor.clone());
                                            init = true;
                                            break;
                                        }
                                    }
                                    if !init {
                                        inputs_raw.insert(name.clone(), tensor.clone());
                                    }
                                }
                                stdin.write(&inputs_raw);

                                // Write inputs hashes
                                stdin
                                    .write(&inference_data_hashes.get(&inference_id).unwrap()[mid]);

                                // Write node
                                stdin.write(&node);

                                info!("Using the network SP1 prover.");
                                let client = ProverClient::builder().network().build();
                                info!(
                                    "Executing the SP1 program. Proving ONNX operator: {:?}",
                                    node
                                );

                                let (public_values, report) =
                                    client.execute(ELF, &stdin).run().unwrap();
                                info!(
                                    "executed program with {} cycles",
                                    report.total_instruction_count()
                                );

                                info!("Raw public values: {:?}", public_values.raw());

                                let (pk, vk) = client.setup(ELF);
                                info!("generated keys (setup)");

                                let program_hash = client.register_program(&vk, ELF).await?;
                                info!("registered program with hash: {:?}", program_hash);

                                let proof = client
                                    .prove(&pk, &stdin)
                                    .cycle_limit(1_000_000_000)
                                    .strategy(FulfillmentStrategy::Hosted)
                                    .skip_simulation(true)
                                    .plonk()
                                    .run()
                                    .unwrap();
                                info!("generated proof");

                                let proof_bytes = proof.bytes();

                                info!(
                                    "Resolving the challenge id {} for operator {} with SP1 proof verification (public values: {}, proof: {})",
                                    challenge_id,
                                    mid,
                                    public_values.raw(),
                                    proof.bytes().encode_hex()
                                );
                                let tx = fault_proof
                                    .resolveOpenChallenge(
                                        challenge_id,
                                        Bytes::from_str(public_values.raw().as_str()).unwrap(),
                                        Bytes::copy_from_slice(&proof_bytes),
                                    )
                                    .send()
                                    .await?;
                                info!("Transaction hash: {}", tx.tx_hash());
                            }
                            (false, false) | (false, true) => {
                                // Move left
                                high = mid - 1;
                                mid = (low + high) / 2;
                                let (input_data_hash, output_data_hash) =
                                    inference_hashes.get(&inference_id).unwrap()[mid];
                                info!(
                                    "Operator execution for operator {} proposed with input data hash: {:?}, output data hash: {:?}",
                                    mid,
                                    input_data_hash.encode_hex(),
                                    output_data_hash.encode_hex()
                                );
                                let tx = fault_proof
                                    .proposeOperatorExecution(
                                        challenge_id,
                                        input_data_hash.into(),
                                        output_data_hash.into(),
                                    )
                                    .send()
                                    .await?;
                                info!("Transaction hash: {}", tx.tx_hash());
                            }
                        }
                    }
                }
                Some(&ChallengeResolved::SIGNATURE_HASH) => {
                    let request = ChallengeResolved::decode_log_data(log.data());
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
        }
    }

    Ok(())
}
