use alloy::{
    eips::BlockNumberOrTag,
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{Provider, ProviderBuilder, WsConnect},
    rpc::types::Filter,
    signers::local::LocalSigner,
    sol,
    sol_types::SolEvent,
};
use futures_util::stream::StreamExt;
use std::str::FromStr;
use tracing::info;

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

    /// Whether to submit correct result (for testing purposes)
    #[clap(long)]
    pub correct: bool,
}

sol!(
    #[derive(Debug)]
    event InferenceRequested(uint256 modelId, uint256 requestId, bytes input);
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

    while let Some(log) = stream.next().await {
        // Parse the data
        println!("Received inference request: {:?}", log);
        let request = InferenceRequested::decode_log_data(log.data(), false);
        if request.is_err() {
            info!("Failed to decode the request data");
            continue;
        }
        let request = request.unwrap();
        info!("Decoded request: {:?}", request);
        let model_id: U256 = request.modelId;
        let inference_id: U256 = request.requestId;
        let input_data_arr_u8 = request.input.as_ref();
        let num_f64s = input_data_arr_u8.len() / 8;
        let input_data = vec![unsafe {
            let f64_slice =
                std::slice::from_raw_parts(input_data_arr_u8.as_ptr() as *const f64, num_f64s);
            f64_slice.to_vec()
        }];
        info!(
            "Model id: {}, Inference id: {}, Input data: {:?}",
            model_id, inference_id, input_data
        );

        // Perform the inference

        // Submit the result

        // TODO: spawn new thread and check if someone will challenge the result
    }

    Ok(())
}
