use minusagent::config::Config;
use minusagent::transport::cli::Cli;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => {
            println!("No config found. Creating default at ~/.minusagent/config.json");
            match Config::init() {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to create config: {}", e);
                    std::process::exit(1);
                }
            }
        }
    };

    let mut cli = Cli::new(config);
    cli.run().await;
}
