# dravr-commere

Multi-tenant push notification service with dispatch pipeline, scheduling, and Expo Push delivery.

## Architecture

```
dravr-commere          Core library (models, service, dispatch, triggers, scheduler)
dravr-commere-mcp      MCP server (JSON-RPC 2.0 over stdio/HTTP)
dravr-commere-server   Unified REST API + MCP server
```

## Development

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
```

## License

MIT OR Apache-2.0
