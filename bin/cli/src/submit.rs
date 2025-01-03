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
use candle_core::{Device, Tensor};
use futures_util::stream::StreamExt;
use std::{collections::HashMap, str::FromStr};
use tracing::info;
use zkopml_ml::onnx::load_onnx_model;

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
    event InferenceRequested(uint256 modelId, uint256 requestId, bytes inputData);
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
        .event("InferenceRequested(uint256,uint256,bytes)")
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
        let inference_id: U256 = request.requestId;
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
        let input = Tensor::from_vec(input_data, input_shape.clone(), &Device::Cpu)?;
        let mut inputs: HashMap<String, Tensor> = HashMap::new();
        inputs.insert("input".to_string(), input);
        let result = model.inference(&mut inputs)?;
        info!("Inference result: {:?}", result["output"]);

        // Submit the result
        let mut output_data: Vec<f32> = result["output"].flatten_all()?.to_vec1::<f32>()?;
        // If defect flag is set, submit wrong result
        // TODO: figure out how to do defects anywhere in the computation graph of ONNX
        if args.defect {
            info!("Augmenting the result with a defect so it is wrong");
            output_data[0] += 1.0;
        }
        let output_data = Bytes::from_iter(unsafe {
            std::slice::from_raw_parts(output_data.as_ptr() as *const u8, output_data.len() * 4)
                .iter()
        });
        let model_registry = zkopml_contracts::ModelRegistry::new(
            args.model_registry_address,
            user_provider.clone(),
        );
        let tx = model_registry
            .respondInference(inference_id, output_data)
            .send()
            .await?;
        info!("Transaction hash: {}", tx.tx_hash());
        info!("Inference {} responded", inference_id);

        std::thread::sleep(std::time::Duration::from_secs(5));

        // TODO: spawn new thread and check if someone will challenge the result
    }

    Ok(())
}
