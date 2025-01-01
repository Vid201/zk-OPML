pub mod deploy;
pub mod prove;
pub mod register;
pub mod request;
pub mod submit;
pub mod tracing_util;
pub mod verify;

#[derive(clap::Parser, Debug, Clone)]
#[command(name = "zkopml-cli")]
#[command(bin_name = "zkopml-cli")]
#[command(author, version, about, long_about = None)]
pub enum Cli {
    Deploy(deploy::DeployArgs),
    Register(register::RegisterArgs),
    Request(request::RequestArgs),
    Submit(submit::SubmitArgs),
    Verify(verify::VerifyArgs),
    Prove(prove::ProveArgs),
}

impl Cli {
    pub fn verbosity(&self) -> u8 {
        match self {
            Cli::Deploy(args) => args.v,
            Cli::Register(args) => args.v,
            Cli::Request(args) => args.v,
            Cli::Submit(args) => args.v,
            Cli::Verify(args) => args.v,
            Cli::Prove(args) => args.v,
        }
    }
}
