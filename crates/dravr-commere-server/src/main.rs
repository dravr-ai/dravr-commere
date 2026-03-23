// ABOUTME: Unified CLI entry point for dravr-commere server
// ABOUTME: Supports serve (REST+MCP), MCP stdio, and future CLI commands
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::sync::Arc;

use clap::{Parser, Subcommand};
use tracing::info;

use dravr_commere_mcp::state::create_shared_state;
use dravr_commere_mcp::transport::stdio::StdioTransport;
use dravr_commere_mcp::transport::McpTransport;
use dravr_commere_mcp::{build_tool_registry, McpServer};
use dravr_commere_server::router::build_router;

/// Default host
const DEFAULT_HOST: &str = "127.0.0.1";
/// Default port
const DEFAULT_PORT: u16 = 3200;

#[derive(Parser)]
#[command(
    name = "dravr-commere-server",
    version,
    about = "Push notification server"
)]
struct Cli {
    /// Transport mode for MCP (when no subcommand)
    #[arg(long, default_value = "http")]
    transport: String,

    /// HTTP host
    #[arg(long)]
    host: Option<String>,

    /// HTTP port
    #[arg(long)]
    port: Option<u16>,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Start the REST + MCP HTTP server
    Serve {
        /// HTTP host
        #[arg(long)]
        host: Option<String>,
        /// HTTP port
        #[arg(long)]
        port: Option<u16>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();

    let state = create_shared_state();
    let tools = build_tool_registry();
    let mcp_server = Arc::new(McpServer::new(state, tools));

    match cli.command {
        Some(Command::Serve { host, port }) => {
            let host = host.unwrap_or_else(|| DEFAULT_HOST.to_owned());
            let port = port.unwrap_or(DEFAULT_PORT);
            serve_http(mcp_server, &host, port).await?;
        }
        None => {
            let host = cli.host.unwrap_or_else(|| DEFAULT_HOST.to_owned());
            let port = cli.port.unwrap_or(DEFAULT_PORT);
            if cli.transport == "stdio" {
                StdioTransport.serve(mcp_server).await?;
            } else {
                serve_http(mcp_server, &host, port).await?;
            }
        }
    }

    Ok(())
}

async fn serve_http(
    mcp_server: Arc<McpServer>,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = build_router(mcp_server);
    let addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("dravr-commere server listening on {addr}");
    info!("  Health: GET http://{addr}/health");
    info!("  MCP:    POST http://{addr}/mcp");

    axum::serve(listener, app).await?;
    Ok(())
}
