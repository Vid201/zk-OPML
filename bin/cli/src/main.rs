use clap::Parser;
use zkopml_cli::tracing_util::init_tracing_subscriber;
use zkopml_cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    init_tracing_subscriber(cli.verbosity())?;

    match cli {
        Cli::Deploy(args) => zkopml_cli::deploy::deploy(args).await?,
        Cli::Register(args) => zkopml_cli::register::register(args).await?,
        Cli::Request(args) => zkopml_cli::request::request(args).await?,
        Cli::Submit(args) => zkopml_cli::submit::submit(args).await?,
        Cli::Verify(args) => zkopml_cli::verify::verify(args).await?,
        Cli::Prove(args) => zkopml_cli::prove::prove(args).await?,
    }

    Ok(())
}
