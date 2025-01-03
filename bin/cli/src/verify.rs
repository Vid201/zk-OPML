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
use futures_util::StreamExt;
use std::{collections::HashMap, str::FromStr};
use tracing::info;
use zkopml_ml::onnx::load_onnx_model;

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
}

sol!(
    #[derive(Debug)]
    event InferenceResponded(
        uint256 modelId,
        uint256 inferenceId,
        bytes outputData
    );
);

pub async fn verify(args: VerifyArgs) -> anyhow::Result<()> {
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

    // Listen for inference responses
    let _model_registry =
        zkopml_contracts::ModelRegistry::new(args.model_registry_address, user_provider.clone());
    let inference_response_filter = Filter::new()
        .address(args.model_registry_address)
        .event("InferenceResponded(uint256,uint256,bytes)")
        .from_block(BlockNumberOrTag::Latest);
    let sub = user_provider
        .subscribe_logs(&inference_response_filter)
        .await?;
    let mut stream = sub.into_stream();

    let input_shape = args.input_shape.clone();

    while let Some(log) = stream.next().await {
        // Parse the data
        info!("Received inference response: {:?}", log);
        let response = InferenceResponded::decode_log_data(log.data(), false);
        if response.is_err() {
            info!("Failed to decode the response data");
            continue;
        }
        let response = response.unwrap();
        info!("Decoded response: {:?}", response);
        let model_id: U256 = response.modelId;
        let inference_id: U256 = response.inferenceId;
        let output_data_arr_u8 = response.outputData.as_ref();
        let num_f32s = output_data_arr_u8.len() / 4;
        let output_data = vec![unsafe {
            let f32_slice =
                std::slice::from_raw_parts(output_data_arr_u8.as_ptr() as *const f32, num_f32s);
            f32_slice.to_vec()
        }];
        info!(
            "Model id: {}, Inference id: {}, Output data: {:?}",
            model_id, inference_id, output_data
        );

        // Get the inference input data
        let model_registry = zkopml_contracts::ModelRegistry::new(
            args.model_registry_address,
            user_provider.clone(),
        );
        let inference = model_registry.getInference(inference_id).call().await?;
        let input_data_arr_u8 = inference.inference.inputData.as_ref();
        let num_f32s = input_data_arr_u8.len() / 4;
        let input_data = vec![unsafe {
            let f32_slice =
                std::slice::from_raw_parts(input_data_arr_u8.as_ptr() as *const f32, num_f32s);
            f32_slice.to_vec()
        }];
        info!("Inference input data: {:?}", input_data);

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

        // Compare the result with the expected output
        let output_data: Vec<f32> = result["output"].flatten_all()?.to_vec1::<f32>()?;
        let output_data = Bytes::from_iter(unsafe {
            std::slice::from_raw_parts(output_data.as_ptr() as *const u8, output_data.len() * 4)
                .iter()
        });
        if output_data == response.outputData {
            info!("Output data matches the expected result, not challenging");
            continue;
        }

        // TODO: act on result not the same - dispute
        info!("Output data does not match the expected result, challenging");
    }

    Ok(())
}
