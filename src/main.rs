use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    minusagent::interface::cli::run().await
}
