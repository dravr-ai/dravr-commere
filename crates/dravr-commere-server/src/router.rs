// ABOUTME: Axum router combining REST API endpoints with MCP handler
// ABOUTME: Routes for health check, MCP protocol, and notification endpoints
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use std::sync::Arc;

use axum::middleware;
use axum::Router;

use dravr_commere_mcp::state::SharedState;

use crate::auth;
use crate::health::health_check;

/// Build the application router with all routes
///
/// Routes:
/// - `GET /health` — Server health check
/// - `POST /mcp` — MCP Streamable HTTP (JSON-RPC 2.0, via dravr-tronc)
///
/// The auth middleware is applied to all routes. It only enforces
/// authentication when `COMMERE_API_TOKEN` is set.
pub fn build_router(state: SharedState) -> Router {
    let mcp_server = Arc::new(dravr_tronc::McpServer::new(
        "dravr-commere",
        env!("CARGO_PKG_VERSION"),
        dravr_commere_mcp::build_tool_registry(),
        Arc::clone(&state),
    ));

    let mcp_router = dravr_tronc::mcp::transport::http::mcp_router(mcp_server);

    Router::new()
        .route("/health", axum::routing::get(health_check))
        .with_state(state)
        .merge(mcp_router)
        .layer(middleware::from_fn(auth::require_auth))
}
