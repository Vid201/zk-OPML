use alloy::{network::EthereumWallet, providers::ProviderBuilder, signers::local::LocalSigner};
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
    let _owner_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&owner_wallet)
        .on_http(args.eth_node_address.as_str().try_into()?);
    // TODO: use owner provider

    // Initialize the deployer wallet
    info!("Initializing deployer wallet.");
    let deployer_signer = LocalSigner::from_str(&args.deployer_key)?;
    let deployer_wallet = EthereumWallet::from(deployer_signer);
    let deployer_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(&deployer_wallet)
        .on_http(args.eth_node_address.as_str().try_into()?);

    // Deploy ModelRegistry contract
    info!("Deploying ModelRegistry contract.");
    let model_registry_contract = zkopml_contracts::ModelRegistry::deploy_builder(&deployer_provider)
        .await
        .context("ModelRegistry contract deployment failed")?;
    info!("{:?}", &model_registry_contract);

    // TODO: deploy zkVM verifier contracts
    // TODO: deploy DisputeGameContracts

    Ok(())
}
