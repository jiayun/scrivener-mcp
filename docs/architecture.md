# Architecture & Module Design

## Overview

scrivener-mcp is a Rust MCP (Model Context Protocol) server binary that integrates `scrivener` (project I/O) and `writing-analysis` (text analysis) crates, exposing Scrivener 3 project operations as MCP tools via the `rmcp` SDK. It enables AI assistants like Claude Desktop to read, write, search, and analyze Scrivener projects through a standardized protocol.

> Inspired by [dcondrey/scrivener-mcp](https://github.com/dcondrey/scrivener-mcp) (MIT License, Copyright 2025 David Condrey)

## Data Flow

```
MCP Client (Claude Desktop / Cursor / etc.)
    │
    ▼
rmcp (stdio transport, JSON-RPC 2.0)
    │
    ▼
ScrivenerMcp (ServerHandler)
    │
    ├── tool_router dispatch
    │
    ▼
Handlers
    ├── ProjectHandlers  ──▶  scrivener::Project (open/save/structure)
    ├── DocumentHandlers ──▶  scrivener::Document (read/write/metadata)
    ├── SearchHandlers   ──▶  scrivener::Project (search/trash)
    ├── CompileHandlers  ──▶  scrivener::Project (compile/stats)
    ├── AnalysisHandlers ──▶  writing_analysis (readability/passive/sentiment)
    └── MemoryHandlers   ──▶  rusqlite (project memory/session state)
```

### Request Lifecycle

```
JSON-RPC request (stdin)
    │
    ▼
rmcp deserialize ──▶ ServerHandler::call_tool()
    │
    ▼
ToolRouter dispatch by tool name
    │
    ▼
Handler fn: validate params → acquire state lock → call crate API → build response
    │
    ▼
CallToolResult (Content::text / Content::json) ──▶ JSON-RPC response (stdout)
```

## Module Structure

```
src/
├── main.rs             # Entry point: CLI args, tracing setup, stdio serve
├── server.rs           # ScrivenerMcp struct, all tool definitions, ServerHandler impl
├── services/
│   ├── mod.rs          # Service module re-exports
│   ├── project.rs      # ProjectSession struct
│   └── database.rs     # SQLite connection and queries
├── types.rs            # MCP-specific types, tool input/output structs
└── error.rs            # McpServerError enum, error mapping

tests/
├── integration/
│   ├── project_tools_test.rs
│   ├── document_tools_test.rs
│   ├── search_tools_test.rs
│   ├── analysis_tools_test.rs
│   └── memory_tools_test.rs
└── fixtures/
    └── sample.scriv/        # Symlink or copy from scrivener-rs fixtures
```

### Module Responsibilities

| Module | Visibility | Responsibility |
|--------|-----------|----------------|
| `main.rs` | binary | CLI arg parsing (clap), tracing init, stdio transport, server start |
| `server.rs` | internal | `ScrivenerMcp` struct with all 29 tools via multiple `#[tool_router]` blocks, `ServerHandler` impl |
| `services/project.rs` | internal | `ProjectSession` struct definition |
| `services/database.rs` | internal | SQLite connection, memory/cache/session queries |
| `types.rs` | internal | Tool input/output types with `serde` + `schemars` derives |
| `error.rs` | internal | `McpServerError` enum, `From` impls for upstream errors |

## Key Design Decisions

### 1. rmcp `#[tool_router]` Macro

Each handler group is a separate struct with `#[tool_router]` on its `impl` block. Individual tools are annotated with `#[tool(name = "...", description = "...")]`. The macro auto-generates `list_tools` and `call_tool` dispatch.

```rust
#[tool_router]
impl ProjectHandlers {
    #[tool(name = "open_project", description = "Open a Scrivener project")]
    async fn open_project(&self, params: Parameters<OpenProjectParams>) -> Result<CallToolResult, McpError> {
        // ...
    }
}
```

### 2. Handler Group Composition

Multiple `#[tool_router(router = name)]` blocks on the same `ScrivenerMcp` struct, each generating a named router factory. Combined with the `+` operator:

```rust
let tool_router = Self::project_router()
    + Self::document_router()
    + Self::search_router()
    + Self::compile_router()
    + Self::analysis_router()
    + Self::memory_router()
    + Self::stats_router();
```

The `#[tool_handler(router = self.tool_router)]` macro on `ServerHandler` delegates `list_tools`/`call_tool` to the combined router.

### 3. Session-based Project Management

Only one Scrivener project is open at a time, stored in `Arc<Mutex<Option<ProjectSession>>>`. Tools that require an open project acquire the lock and return an error if no project is open.

```rust
pub struct ProjectSession {
    pub project: scrivener::Project,
    pub project_path: PathBuf,
    pub opened_at: chrono::DateTime<chrono::Utc>,
}
```

Rationale: Scrivener projects are filesystem bundles — concurrent access to multiple projects adds complexity with minimal benefit for typical AI assistant use cases.

### 4. SQLite for Persistence

`rusqlite` stores:
- **Project memory**: AI-generated notes, character profiles, plot summaries (survives across sessions)
- **Analysis cache**: Cached analysis results keyed by document UUID + content hash
- **Session history**: When projects were opened/closed, for context continuity

The database file is stored at `~/.scrivener-mcp/data.db` by default, configurable via CLI args.

### 5. Error Mapping

Upstream errors from `scrivener` and `writing_analysis` are mapped to `rmcp::ErrorData`:

```rust
fn map_to_mcp_error(e: impl std::error::Error) -> McpError {
    McpError {
        code: ErrorCode::INTERNAL_ERROR,
        message: e.to_string().into(),
        data: None,
    }
}
```

### 6. Stdio Transport (Default)

The server uses stdio transport (stdin/stdout) by default, which is the standard for Claude Desktop integration. The `rmcp` SDK handles JSON-RPC framing over stdio.

```rust
let transport = rmcp::transport::io::stdio();
server.serve(transport).await?;
```

Future: SSE transport can be added for web-based clients without changing the handler logic.

### 7. Tracing for Logging

All logging uses the `tracing` crate with structured fields. Logs go to stderr (not stdout, which is reserved for JSON-RPC). The log level is configurable via `RUST_LOG` env var or `--log-level` CLI arg.

### 8. Tool Parameter Validation via schemars

Tool input types derive `schemars::JsonSchema`, which rmcp uses to generate JSON Schema for `list_tools` responses. This gives MCP clients automatic parameter validation and documentation.

## MCP Tool Summary

| Category | Count | Tools |
|----------|-------|-------|
| Project | 4 | open_project, close_project, refresh_project, get_structure |
| Document | 9 | read_document, write_document, create_document, create_folder, delete_document, rename_document, move_document, get_document_info, update_metadata |
| Search | 4 | search_content, list_trash, search_trash, recover_document |
| Compilation | 3 | compile_documents, export_project, get_statistics |
| Analysis | 3 | analyze_document, get_word_count, analyze_readability |
| Memory | 4 | update_memory, get_memory, check_consistency, get_project_summary |
| Stats | 2 | get_writing_stats, get_session_info |
| **Total** | **29** | |

See `mcp-tools-spec.md` for full tool specifications.

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `rmcp` | 0.1 | MCP SDK — ServerHandler, tool macros, stdio transport |
| `scrivener` | 0.1 | Scrivener 3 project reading/writing |
| `writing-analysis` | 0.1 | Text analysis (readability, passive voice, sentiment) |
| `rusqlite` | 0.32 | SQLite for project memory and analysis cache |
| `tokio` | 1.0 | Async runtime (required by rmcp) |
| `serde` | 1.0 | Serialization/deserialization |
| `serde_json` | 1.0 | JSON handling for tool params/results |
| `schemars` | 0.8 | JSON Schema generation for tool input types |
| `clap` | 4.0 | CLI argument parsing |
| `tracing` | 0.1 | Structured logging |
| `tracing-subscriber` | 0.3 | Log output formatting |
| `chrono` | 0.4 | DateTime handling |
| `thiserror` | 2.0 | Error type derivation |
| `uuid` | 1.12 | UUID handling (re-exported from scrivener) |
| `anyhow` | 1.0 | (dev) Test error handling |
| `tempfile` | 3.8 | (dev) Temporary directories for tests |
