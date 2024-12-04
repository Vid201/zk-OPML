use alloy::{network::EthereumWallet, providers::ProviderBuilder, signers::local::LocalSigner};
use zkopml_ml::onnx::load_onnx_model;
use std::str::FromStr;
use tracing::info;

#[derive(clap::Args, Debug, Clone)]
pub struct RegisterArgs {
    #[arg(long, short, help = "Verbosity level (0-4)", action = clap::ArgAction::Count)]
    pub v: u8,

    /// Address of the Ethereum node endpoint to use
    #[clap(long)]
    pub eth_node_address: String,

    /// Secret key to use for registering the model
    #[clap(long)]
    pub user_key: String,

    /// Path to the model file (ONNX)
    #[clap(long)]
    pub model_path: String,
}

pub async fn register(args: RegisterArgs) -> anyhow::Result<()> {
    // Initialize the user wallet
    info!("Initializing user wallet.");
    let user_signer = LocalSigner::from_str(&args.user_key)?;
    let user_wallet = EthereumWallet::from(user_signer);
    let _user_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&user_wallet)
        .on_http(args.eth_node_address.as_str().try_into()?);

    // Read the model file
    let mut file = std::fs::File::open(args.model_path)?;
    let model = load_onnx_model(&mut file)?;

    for node in model.graph.nodes() {
        let node_str = format!("{:?}", node);
        println!("{}", node_str);
        println!("Hash: {}", zkopml_ml::merkle::hash_string(&node_str));
    }

    // Create merkle tree from ONNX operators

    // Publish the model to the decentralized storage

    // Publish the model metadata to the ModelRegistry contract

    Ok(())
}
