use alloy::{
    network::EthereumWallet,
    providers::{ProviderBuilder, WsConnect},
    signers::local::LocalSigner,
};
use anyhow::Context;
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
}

pub async fn deploy(args: DeployArgs) -> anyhow::Result<()> {
    // Initialize the owner wallet
    info!("Initializing owner wallet.");
    let owner_signer = LocalSigner::from_str(&args.owner_key)?;
    let owner_wallet = EthereumWallet::from(owner_signer);
    let ws_connect = WsConnect::new(args.eth_node_address);
    let _owner_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&owner_wallet)
        .on_ws(ws_connect.clone())
        .await?;
    info!("Owner address: {}", owner_wallet.default_signer().address());
    // TODO: use owner provider

    // Initialize the deployer wallet
    info!("Initializing deployer wallet.");
    let deployer_signer = LocalSigner::from_str(&args.deployer_key)?;
    let deployer_wallet = EthereumWallet::from(deployer_signer);
    let deployer_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(deployer_wallet.clone())
        .on_ws(ws_connect)
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
    info!("{:?}", &model_registry_contract.address());

    // TODO: deploy DisputeGameContracts

    Ok(())
}
