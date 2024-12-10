pub mod deploy;
pub mod register;
pub mod request;
pub mod tracing_util;

#[derive(clap::Parser, Debug, Clone)]
#[command(name = "zkopml-cli")]
#[command(bin_name = "zkopml-cli")]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    Deploy(deploy::DeployArgs),
    Register(register::RegisterArgs),
    Request(request::RequestArgs),
}

impl Cli {
    pub fn verbosity(&self) -> u8 {
        match self {
            Cli::Deploy(args) => args.v,
            Cli::Register(args) => args.v,
            Cli::Request(args) => args.v,
        }
    }
}
