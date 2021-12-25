use anyhow::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    dotenv().ok();

    esa_cli::run().await
}
