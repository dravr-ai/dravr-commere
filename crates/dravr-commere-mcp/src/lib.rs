// ABOUTME: MCP server library exposing notification service via Model Context Protocol
// ABOUTME: Delegates protocol, server, and transport to dravr-tronc; provides tools and state
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

//! # dravr-commere-mcp
//!
//! MCP (Model Context Protocol) server for the dravr-commere notification service.
//! Exposes notification operations as MCP tools via JSON-RPC 2.0.
//!
//! ## Re-exports
//!
//! - [`McpServer`] — JSON-RPC request dispatcher (from dravr-tronc)
//! - [`ServerState`] / [`SharedState`] — shared server state
//! - [`build_tool_registry`] — default tool registry with all notification tools

/// Shared server state
pub mod state;
/// MCP tool definitions and registry
pub mod tools;

pub use dravr_tronc::McpServer;
pub use state::{ServerState, SharedState};
pub use tools::build_tool_registry;
