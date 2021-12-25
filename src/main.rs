use anyhow::Result;
use dotenv::dotenv;
use env_logger;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    dotenv().ok();

    esa_cli::run().await
}
