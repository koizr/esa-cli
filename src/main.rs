extern crate esa_cli;

use anyhow::Result;
use dotenv::dotenv;

use crate::esa_cli::cli;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    cli::run().await
}
