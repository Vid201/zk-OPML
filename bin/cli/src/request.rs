use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, U256},
    providers::ProviderBuilder,
    signers::local::LocalSigner,
};
use std::{fs::File, path::PathBuf, str::FromStr};
use tracing::info;
use zkopml_ml::data::DataFile;

#[derive(clap::Args, Debug, Clone)]
pub struct RequestArgs {
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

    /// Path to the input data
    #[clap(long)]
    pub input_data_path: String,
}

pub async fn request(args: RequestArgs) -> anyhow::Result<()> {
    // Initialize the user wallet
    info!("Initializing user wallet.");
    let user_signer = LocalSigner::from_str(&args.user_key)?;
    let user_wallet = EthereumWallet::from(user_signer);
    let user_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&user_wallet)
        .on_http(args.eth_node_address.as_str().try_into()?);
    info!("User address: {}", user_wallet.default_signer().address());

    // Read the input data
    info!("Reading the input data from {}", args.input_data_path);
    let path = PathBuf::from(&args.input_data_path);
    let reader = File::open(&path)?;
    let data_file: DataFile = serde_json::from_reader(reader)?;
    info!("Input data: {:?}", data_file.input_data);

    // Request the inference
    let model_registry =
        zkopml_contracts::ModelRegistry::new(args.model_registry_address, user_provider);
    let model_id = U256::from(args.model_id);
    let input_data = Bytes::from_iter(unsafe {
        std::slice::from_raw_parts(
            data_file.input_data.as_ptr() as *const u8,
            data_file.input_data.len() * 8,
        )
        .iter()
    });
    let tx_hash = model_registry
        .requestInference(model_id, input_data)
        .send()
        .await?
        .watch()
        .await?;
    info!("Transaction hash: {}", tx_hash);
    let inference_id = U256::from(model_registry.inferenceCounter().call().await?._0);
    info!(
        "Inference request sent with id: {}",
        inference_id - U256::from(1)
    );

    // TODO: wait and listen for result

    Ok(())
}
