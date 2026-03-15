# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Rust MCP server exposing 28 tools for AI assistants to interact with Scrivener 3 writing projects. Uses stdio transport (JSON-RPC over stdin/stdout). Single binary: `scrivener-mcp`.

## Build & Development Commands

```bash
cargo build                    # Build
cargo run                      # Run server (stdio transport)
cargo test                     # Run all tests
cargo test test_name           # Run a single test
cargo clippy -- -D warnings    # Lint (CI uses deny warnings)
cargo fmt --check              # Check formatting
cargo fmt                      # Auto-format
```

**CLI options:** `--db_path <PATH>` (default: `~/.scrivener-mcp/data.db`), `--log_level <LEVEL>` (default: `info`)

## Architecture

**Entry flow:** `main.rs` (CLI parsing, tracing init, stdio server start) → `server.rs` (ScrivenerMcp struct, all 28 tool handlers) → `services/` (persistence layer)

**Key modules:**
- `server.rs` — Core server with tool routing via `rmcp` macros. Tools are grouped into 7 router blocks (project, document, search, compile, analysis, memory, stats) composed with `+` operator. Each tool has a public wrapper returning `String` and a private `do_*` implementation returning `Result<String>`.
- `types.rs` — All tool parameter/response types. Derives `Serialize`, `Deserialize`, `JsonSchema` for auto MCP schema generation.
- `error.rs` — `McpServerError` enum wrapping upstream errors, mapped to `rmcp::ErrorData`.
- `services/database.rs` — SQLite persistence: `project_memory`, `analysis_cache`, `session_history` tables.
- `services/project.rs` — `ProjectSession` struct holding the open `scrivener::Project`.

**State management:** Single project open at a time via `Arc<Mutex<Option<ProjectSession>>>`. Database connection in `Arc<Database>` with `std::sync::Mutex` (sync, not tokio).

**Key crates:**
- `rmcp` — MCP SDK (server, transport, macros)
- `scrivener` — Scrivener 3 project I/O (published crate)
- `writing-analysis` — Rule-based text analysis (readability, passive voice, sentiment)
- `rusqlite` — SQLite with bundled driver

## CI/CD

CI runs on push to `main` and PRs: clippy (deny warnings), test, fmt check. Release workflow triggers on `v*` tags, building for macOS (Intel + Apple Silicon), Linux, Windows.

## Design Decisions

- No AI processing in the server — only data access and rule-based analysis; AI generation is the client's responsibility.
- All tool handlers are request-response (no async job queue).
- Project memory persists in SQLite keyed by project path, surviving across sessions.
- Logs go to stderr (stdout is reserved for MCP JSON-RPC protocol).
