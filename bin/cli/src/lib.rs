pub mod deploy;
pub mod tracing_util;

#[derive(clap::Parser, Debug, Clone)]
#[command(name = "zkopml-cli")]
#[command(bin_name = "zkopml-cli")]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    Deploy(deploy::DeployArgs),
}

impl Cli {
    pub fn verbosity(&self) -> u8 {
        match self {
            Cli::Deploy(args) => args.v,
        }
    }
}
