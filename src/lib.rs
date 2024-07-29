pub mod api;
mod cli;
pub mod utils;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    cli::run().await;
    Ok(())
}
