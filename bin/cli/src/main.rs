use clap::Parser;
use zkopml_cli::tracing_util::init_tracing_subscriber;
use zkopml_cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing_subscriber(cli.verbosity())?;

    match cli {
        Cli::Deploy(args) => zkopml_cli::deploy::deploy(args).await?,
    }

    Ok(())
}
