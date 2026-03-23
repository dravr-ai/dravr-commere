// ABOUTME: CLI entry point for the dravr-commere MCP server
// ABOUTME: Supports stdio and HTTP transport modes via --transport flag
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::sync::Arc;

use clap::Parser;
use dravr_tronc::server::cli::McpArgs;
use tokio::sync::RwLock;
use tracing::info;

use dravr_commere_mcp::state::ServerState;

/// dravr-commere-mcp — MCP server exposing push notification operations
#[derive(Parser)]
#[command(name = "dravr-commere-mcp", version, about)]
struct Cli {
    #[command(flatten)]
    server: McpArgs,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    dravr_tronc::server::tracing_init::init(&cli.server.transport);

    let state = Arc::new(RwLock::new(ServerState::new()));
    let registry = dravr_commere_mcp::build_tool_registry();
    let server = Arc::new(dravr_tronc::McpServer::new(
        "dravr-commere-mcp",
        env!("CARGO_PKG_VERSION"),
        registry,
        state,
    ));

    info!(
        transport = %cli.server.transport,
        "Starting dravr-commere MCP server"
    );

    match cli.server.transport.as_str() {
        "stdio" => dravr_tronc::mcp::transport::stdio::run(server).await?,
        "http" => {
            dravr_tronc::mcp::transport::http::serve(server, &cli.server.host, cli.server.port)
                .await?;
        }
        other => {
            eprintln!("Unknown transport: {other}. Use 'stdio' or 'http'.");
            std::process::exit(1);
        }
    }

    Ok(())
}
