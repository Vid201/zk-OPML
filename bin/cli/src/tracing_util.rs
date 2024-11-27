use anyhow::{anyhow, Result};
use tracing::Level;

pub fn init_tracing_subscriber(verbosity_level: u8) -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(match verbosity_level {
            0 => Level::INFO,
            1 => Level::DEBUG,
            _ => Level::TRACE,
        })
        .finish();
    tracing::subscriber::set_global_default(subscriber).map_err(|e| anyhow!(e))
}
