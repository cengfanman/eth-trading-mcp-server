mod config;
mod eth;
mod rpc;
mod tools;
mod types;
mod uniswap;
mod utils;

use anyhow::Result;
use std::io::{self, BufRead, Write};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    config::Config,
    eth::provider,
    rpc::MCPServer,
    types::MCPRequest,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Load .env file if present
    dotenv::dotenv().ok();

    // Initialize tracing (logs to stderr to not interfere with stdio MCP communication)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    info!("Starting Ethereum Trading MCP Server");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded");

    // Create Ethereum provider and signer
    let provider = provider::create_provider(&config.rpc_url)?;
    let signer = provider::create_signer(&config.rpc_url, &config.private_key, config.chain_id)?;

    // Show wallet address
    use ethers::signers::Signer;
    let wallet_address = signer.address();
    info!("Wallet address: {:?}", wallet_address);

    // Verify connection
    let block_number = provider::get_block_number(&provider).await?;
    info!("Connected to Ethereum network, block: {}", block_number);

    // Create MCP server
    let server = MCPServer::new(provider, signer, config);

    info!("MCP server ready, listening on stdin/stdout");

    // MCP communication loop (stdin/stdout)
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        match line {
            Ok(line) => {
                if line.trim().is_empty() {
                    continue;
                }

                // Parse JSON-RPC request
                match serde_json::from_str::<MCPRequest>(&line) {
                    Ok(request) => {
                        // Handle request
                        let response = server.handle_request(request).await;

                        // Send response
                        let response_json = serde_json::to_string(&response)?;
                        writeln!(stdout, "{}", response_json)?;
                        stdout.flush()?;
                    }
                    Err(e) => {
                        error!("Failed to parse request: {}", e);
                        // Send error response
                        let error_response = types::MCPResponse {
                            jsonrpc: "2.0".to_string(),
                            id: serde_json::Value::Null,
                            result: None,
                            error: Some(types::MCPError {
                                code: -32700,
                                message: "Parse error".to_string(),
                                data: Some(serde_json::json!({"details": e.to_string()})),
                            }),
                        };
                        let response_json = serde_json::to_string(&error_response)?;
                        writeln!(stdout, "{}", response_json)?;
                        stdout.flush()?;
                    }
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }

    info!("MCP server shutting down");
    Ok(())
}
