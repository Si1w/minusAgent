use anyhow::Result;
use clap::Parser;

use minusagent::cli::cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    Cli::parse().run().await
}
