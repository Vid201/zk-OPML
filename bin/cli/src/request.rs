use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, U256},
    providers::{ProviderBuilder, WsConnect},
    signers::local::LocalSigner,
};
use candle_core::Tensor;
use candle_onnx::eval::get_tensor;
use sha2::Digest;
use std::{collections::HashMap, str::FromStr};
use tracing::info;
use zkopml_ml::{data::tensor_hash, onnx::load_onnx_model};

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

    /// Path to the model file (ONNX)
    #[clap(long)]
    pub model_path: String,

    /// Secret key to use for requesting the inference
    #[clap(long)]
    pub user_key: String,

    /// Model id to use
    #[clap(long)]
    pub model_id: u8,
}

pub async fn request(args: RequestArgs) -> anyhow::Result<()> {
    // Initialize the user wallet
    info!("Initializing user wallet.");
    let user_signer = LocalSigner::from_str(&args.user_key)?;
    let user_wallet = EthereumWallet::from(user_signer);
    let ws_connect = WsConnect::new(args.eth_node_address);
    let user_provider = ProviderBuilder::new()
        .wallet(&user_wallet)
        .connect_ws(ws_connect)
        .await?;
    info!("User address: {}", user_wallet.default_signer().address());

    // Read the model just to structure the input data
    // TODO: read the model file from IPFS based on model id
    // For now, we are going to assume there is only one model
    info!("Reading the model file from {}", args.model_path);
    let model_path = args.model_path.clone();
    let model = load_onnx_model(&model_path)?;
    let mut inputs: HashMap<String, Tensor> = HashMap::new();
    model.prepare_inputs(&mut inputs)?;
    for t in model.graph().clone().unwrap().initializer.iter() {
        let tensor = get_tensor(t, t.name.as_str())?;
        inputs.insert(t.name.to_string(), tensor);
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
    let node = model.get_node(0).unwrap();
    inputs.retain(|k: &String, _| node.input.contains(k));

    // Request the inference
    let model_registry =
        zkopml_contracts::ModelRegistry::new(args.model_registry_address, user_provider);
    let model_id = U256::from(args.model_id);
    let input_data = Bytes::copy_from_slice(serde_json::to_string(&inputs).unwrap().as_bytes());

    let tx = model_registry
        .requestInference(model_id, input_data, hash.into())
        .send()
        .await?;
    info!("Transaction hash: {}", tx.tx_hash());
    std::thread::sleep(std::time::Duration::from_secs(10));
    let inference_id = model_registry.inferenceCounter().call().await? - U256::from(1);
    info!("Inference request sent with id: {}", inference_id);

    // TODO: wait and listen for result

    Ok(())
}
