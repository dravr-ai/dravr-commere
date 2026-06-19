// ABOUTME: Unified CLI entry point for dravr-commere server
// ABOUTME: Supports serve (REST+MCP) via HTTP or MCP-only via stdio
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::error::Error;
use std::process;
use std::sync::Arc;

use clap::Parser;
use dravr_tronc::mcp::transport::stdio;
use dravr_tronc::server::cli::ServerArgs;
use dravr_tronc::server::tracing_init;
use dravr_tronc::McpServer;
use tokio::net::TcpListener;
use tracing::info;

use dravr_commere_mcp::state::ServerState;
use dravr_commere_server::router::build_router;

/// dravr-commere-server — Push notification server (REST API + MCP)
#[derive(Parser)]
#[command(name = "dravr-commere-server", version, about)]
struct Cli {
    #[command(flatten)]
    server: ServerArgs,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let cli = Cli::parse();
    tracing_init::init_with_notifications(&cli.server.transport);

    let state = Arc::new(ServerState::new());

    info!(
        transport = %cli.server.transport,
        "Starting dravr-commere server"
    );

    match cli.server.transport.as_str() {
        "stdio" => {
            let mcp_server = Arc::new(McpServer::new(
                "dravr-commere",
                env!("CARGO_PKG_VERSION"),
                dravr_commere_mcp::build_tool_registry(),
                Arc::clone(&state),
            ));
            stdio::run(mcp_server).await?;
        }
        "http" => {
            let app = build_router(state);
            let addr = format!("{}:{}", cli.server.host, cli.server.port);
            let listener = TcpListener::bind(&addr).await?;

            info!(address = %addr, "HTTP transport listening");
            info!("  Health: GET http://{addr}/health");
            info!("  MCP:    POST http://{addr}/mcp");

            axum::serve(listener, app).await?;
        }
        other => {
            eprintln!("Unknown transport: {other}. Use 'http' or 'stdio'.");
            process::exit(1);
        }
    }

    Ok(())
}
