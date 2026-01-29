use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    minusagent::cli::run().await
}
