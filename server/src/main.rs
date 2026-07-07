mod claude_cli;
mod config;
mod error;
mod gate;
mod prompts;
mod server;
mod telemetry;

use rmcp::{transport::stdio, ServiceExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = config::Config::from_env();
    let service = server::DspecServer { cfg }.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}