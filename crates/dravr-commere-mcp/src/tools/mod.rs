// ABOUTME: MCP tool registry for push notification service tools
// ABOUTME: Delegates McpTool trait and ToolRegistry to dravr-tronc, registers commere-specific tools
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 dravr.ai

use dravr_tronc::mcp::tool::ToolRegistry;

use crate::state::ServerState;

/// Build the default tool registry with all available notification tools
#[must_use]
pub fn build_tool_registry() -> ToolRegistry<ServerState> {
    ToolRegistry::new()
}
