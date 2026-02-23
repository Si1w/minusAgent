use anyhow::Result;
use clap::Parser;

use minusagent::interface::cli::{Cli, Commands, init_config};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Init) => init_config(),
        None => {
            let _session = cli.create_session()?;
            Ok(())
        }
    }
}
