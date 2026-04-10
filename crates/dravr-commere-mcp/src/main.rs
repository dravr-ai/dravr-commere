// ABOUTME: CLI entry point for the dravr-commere MCP server
// ABOUTME: Supports stdio and HTTP transport modes via --transport flag
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::error::Error;
use std::process;
use std::sync::Arc;

use clap::Parser;
use dravr_tronc::mcp::transport::{http, stdio};
use dravr_tronc::server::cli::McpArgs;
use dravr_tronc::server::tracing_init;
use dravr_tronc::McpServer;
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
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();
    tracing_init::init(&cli.server.transport);

    let state = Arc::new(RwLock::new(ServerState::new()));
    let registry = dravr_commere_mcp::build_tool_registry();
    let server = Arc::new(McpServer::new(
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
        "stdio" => stdio::run(server).await?,
        "http" => {
            http::serve(server, &cli.server.host, cli.server.port).await?;
        }
        other => {
            eprintln!("Unknown transport: {other}. Use 'stdio' or 'http'.");
            process::exit(1);
        }
    }

    Ok(())
}
