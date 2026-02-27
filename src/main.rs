use anyhow::Result;
use clap::Parser;

use minusagent::interface::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    Cli::parse().run().await
}
