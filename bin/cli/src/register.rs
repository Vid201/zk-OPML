use alloy::{
    network::EthereumWallet, primitives::Address, providers::ProviderBuilder,
    signers::local::LocalSigner,
};
use ipfs_api_backend_hyper::{IpfsApi, IpfsClient};
use std::{fs::File, str::FromStr};
use tracing::info;
use zkopml_ml::{merkle::ModelMerkleTree, onnx::load_onnx_model};

#[derive(clap::Args, Debug, Clone)]
pub struct RegisterArgs {
    #[arg(long, short, help = "Verbosity level (0-4)", action = clap::ArgAction::Count)]
    pub v: u8,

    /// Address of the Ethereum node endpoint to use
    #[clap(long)]
    pub eth_node_address: String,

    /// Address of the ModelRegistry contract
    #[clap(long)]
    pub model_registry_address: Address,

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
    let user_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&user_wallet)
        .on_http(args.eth_node_address.as_str().try_into()?);

    // Read the model file
    info!("Reading the model file from {}", args.model_path);
    let mut file = std::fs::File::open(args.model_path.clone())?;
    let model = load_onnx_model(&mut file)?;

    // Create merkle tree from ONNX operators
    info!("Creating a Merkle tree from the model operators.");
    let nodes: Vec<_> = model.graph.nodes().into_iter().cloned().collect();
    let merkle_tree = ModelMerkleTree::new(nodes);
    info!("Merkle root hash: {}", merkle_tree.root_hash());

    // Publish the model to the decentralized storage (IPFS)
    info!("Publishing the model to the decentralized storage (IPFS).");
    let client = IpfsClient::default();
    let file = File::open(args.model_path)?;
    let result = client.add(file).await?;
    info!("Model published to IPFS with hash: {}", result.hash);

    // Publish the model metadata to the ModelRegistry contract
    info!("Publishing the model metadata to the ModelRegistry contract.");
    let model_registry =
        zkopml_contracts::ModelRegistry::new(args.model_registry_address, user_provider);
    let model_id = model_registry
        .registerModel(format!("ipfs://{}", result.hash), merkle_tree.root().into())
        .call()
        .await?
        .modelId;
    info!("Model registered with ID: {}", model_id);

    Ok(())
}
