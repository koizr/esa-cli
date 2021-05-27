extern crate esa_cli;

use anyhow::Result;
use dotenv::dotenv;
use env_logger;

use crate::esa_cli::cli;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    dotenv().ok();

    cli::run().await
}
