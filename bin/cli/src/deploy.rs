use alloy::{
    hex::ToHexExt,
    network::EthereumWallet,
    primitives::{Address, U256},
    providers::{ProviderBuilder, WsConnect},
    signers::local::LocalSigner,
};
use anyhow::Context;
use sp1_sdk::{HashableKey, Prover, ProverClient, include_elf};
use std::str::FromStr;
use tracing::info;

#[derive(clap::Args, Debug, Clone)]
pub struct DeployArgs {
    #[arg(long, short, help = "Verbosity level (0-4)", action = clap::ArgAction::Count)]
    pub v: u8,

    /// Address of the Ethereum node endpoint to use
    #[clap(long)]
    pub eth_node_address: String,

    /// Secret key to use for deploying contracts
    #[clap(long)]
    pub deployer_key: String,

    /// Secret key that owns the contracts
    #[clap(long)]
    pub owner_key: String,

    /// Address of the SP1 verifier contract
    #[clap(long)]
    pub sp1_verifier_address: Address,

    /// Challenge window for the FaultProof contract
    #[clap(long)]
    pub challenge_window: u64,

    /// Response window for the FaultProof contract
    #[clap(long)]
    pub response_window: u64,
}

const ELF: &[u8] = include_elf!("zkopml-zk");

pub async fn deploy(args: DeployArgs) -> anyhow::Result<()> {
    // Initialize the owner wallet
    info!("Initializing owner wallet.");
    let owner_signer = LocalSigner::from_str(&args.owner_key)?;
    let owner_wallet = EthereumWallet::from(owner_signer);
    let ws_connect = WsConnect::new(args.eth_node_address);
    let _owner_provider = ProviderBuilder::new()
        .wallet(&owner_wallet)
        .connect_ws(ws_connect.clone())
        .await?;
    info!("Owner address: {}", owner_wallet.default_signer().address());
    // TODO: use owner provider

    // Initialize the deployer wallet
    info!("Initializing deployer wallet.");
    let deployer_signer = LocalSigner::from_str(&args.deployer_key)?;
    let deployer_wallet = EthereumWallet::from(deployer_signer);
    let deployer_provider = ProviderBuilder::new()
        .wallet(deployer_wallet.clone())
        .connect_ws(ws_connect)
        .await?;
    info!(
        "Deployer address: {}",
        deployer_wallet.default_signer().address()
    );

    // Deploy ModelRegistry contract
    info!("Deploying ModelRegistry contract.");
    let model_registry_contract =
        zkopml_contracts::ModelRegistry::deploy(deployer_provider.clone())
            .await
            .context("ModelRegistry contract deployment failed")?;
    info!(
        "ModelRegistry contract deployed at {:?}",
        &model_registry_contract.address()
    );

    // Deploy FaultProof contract
    info!("Deploying FaultProof contract.");
    let client = ProverClient::builder().cpu().build();
    let (_, vk) = client.setup(ELF);
    info!("vk: {:?}", &vk.bytes32_raw().encode_hex());
    let fault_proof_contract = zkopml_contracts::FaultProof::deploy(
        deployer_provider.clone(),
        model_registry_contract.address().clone(),
        U256::from(args.challenge_window),
        U256::from(args.response_window),
        args.sp1_verifier_address,
        vk.bytes32_raw().into(),
    )
    .await
    .context("FaultProof contract deployment failed")?;
    info!(
        "FaultProof contract deployed at {:?}",
        &fault_proof_contract.address()
    );

    Ok(())
}
