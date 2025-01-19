use alloy::{
    network::EthereumWallet,
    primitives::{Address, Bytes, U256},
    providers::{ProviderBuilder, WsConnect},
    signers::local::LocalSigner,
};
use candle_core::Tensor;
use sha2::Digest;
use std::{collections::HashMap, fs::File, path::PathBuf, str::FromStr};
use tracing::info;
use zkopml_ml::{
    data::{tensor_hash, DataFile},
    onnx::load_onnx_model,
};

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

    /// Input shape of the model
    #[clap(long, value_delimiter = ',')]
    pub input_shape: Vec<usize>,

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
    let ws_connect = WsConnect::new(args.eth_node_address);
    let user_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&user_wallet)
        .on_ws(ws_connect)
        .await?;
    info!("User address: {}", user_wallet.default_signer().address());

    // Read the input data
    info!("Reading the input data from {}", args.input_data_path);
    let path = PathBuf::from(&args.input_data_path);
    let reader = File::open(&path)?;
    let data_file: DataFile = serde_json::from_reader(reader)?;
    let input_data: Vec<f32> = data_file.input_data.into_iter().flat_map(|v| v).collect();
    info!("Input data: {:?}", input_data);

    // Read the model just to structure the input data
    // TODO: read the model file from IPFS based on model id
    // For now, we are going to assume there is only one model
    info!("Reading the model file from {}", args.model_path);
    let model_path = args.model_path.clone();
    let model = load_onnx_model(&model_path)?;
    let mut inputs: HashMap<String, Tensor> = HashMap::new();
    model.prepare_inputs(&mut inputs, input_data.clone(), args.input_shape)?;
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

    // Request the inference
    let model_registry =
        zkopml_contracts::ModelRegistry::new(args.model_registry_address, user_provider);
    let model_id = U256::from(args.model_id);
    let input_data = Bytes::from_iter(unsafe {
        std::slice::from_raw_parts(input_data.as_ptr() as *const u8, input_data.len() * 4).iter()
    });

    let tx = model_registry
        .requestInference(model_id, input_data, hash.into())
        .send()
        .await?;
    info!("Transaction hash: {}", tx.tx_hash());
    std::thread::sleep(std::time::Duration::from_secs(10));
    let inference_id = U256::from(model_registry.inferenceCounter().call().await?._0);
    info!(
        "Inference request sent with id: {}",
        inference_id - U256::from(1)
    );

    // TODO: wait and listen for result

    Ok(())
}
