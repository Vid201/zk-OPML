use alloy::{
    hex::ToHexExt, network::EthereumWallet, primitives::{Address, Bytes, U256}, providers::{ProviderBuilder, WsConnect}, signers::local::LocalSigner
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

    /// Input shape of the model
    #[clap(long, value_delimiter = ',')]
    pub input_shape: Vec<usize>,

    /// Output shape of the model
    #[clap(long, value_delimiter = ',')]
    pub output_shape: Vec<usize>,
}

pub async fn register(args: RegisterArgs) -> anyhow::Result<()> {
    // Initialize the user wallet
    info!("Initializing user wallet.");
    let user_signer = LocalSigner::from_str(&args.user_key)?;
    let user_wallet = EthereumWallet::from(user_signer);
    let ws_connect = WsConnect::new(args.eth_node_address);
    let user_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&user_wallet)
        .on_ws(ws_connect)
        .await?;
    info!("User address: {}", user_wallet.default_signer().address());

    // Read the model file
    info!("Reading the model file from {}", args.model_path);
    let model_path = args.model_path.clone();
    let model = load_onnx_model(&model_path)?;

    // Create merkle tree from ONNX operators
    info!("Creating a Merkle tree from the model operators.");
    let nodes = model.graph().unwrap().node;
    let nodes_len = nodes.len();
    let merkle_tree = ModelMerkleTree::new(nodes, model.graph().unwrap());
    info!("Merkle root hash: {:?}", merkle_tree.root().encode_hex());

    // Publish the model to the decentralized storage (IPFS)
    info!("Publishing the model to the decentralized storage (IPFS).");
    let client = IpfsClient::default();
    let file = File::open(model_path)?;
    let result = client.add(file).await?;
    info!("Model published to IPFS with hash: {}", result.hash);

    // Publish the model metadata to the ModelRegistry contract
    info!("Publishing the model metadata to the ModelRegistry contract.");
    let model_registry =
        zkopml_contracts::ModelRegistry::new(args.model_registry_address, user_provider.clone());
    let input_shape = Bytes::from_iter(unsafe {
        std::slice::from_raw_parts(
            args.input_shape.as_ptr() as *const u8,
            args.input_shape.len() * 8,
        )
        .iter()
    });
    let output_shape = Bytes::from_iter(unsafe {
        std::slice::from_raw_parts(
            args.output_shape.as_ptr() as *const u8,
            args.output_shape.len() * 8,
        )
        .iter()
    });
    let tx = model_registry
        .registerModel(
            format!("ipfs://{}", result.hash),
            input_shape,
            output_shape,
            merkle_tree.root().into(),
            U256::from(nodes_len).into(),
        )
        .send()
        .await?;
    info!("Transaction hash: {}", tx.tx_hash());
    std::thread::sleep(std::time::Duration::from_secs(10));
    let model_id = U256::from(model_registry.modelCounter().call().await?._0) - U256::from(1);
    info!("Model registered with ID: {}", model_id);

    Ok(())
}
