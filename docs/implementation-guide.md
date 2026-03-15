# Implementation Guide

## Cargo.toml

```toml
[package]
name = "scrivener-mcp"
version = "0.1.0"
edition = "2021"
license = "MIT"
description = "MCP server for Scrivener 3 projects — AI-powered writing assistant tools"
repository = "https://github.com/jiayun/scrivener-mcp"
keywords = ["mcp", "scrivener", "writing", "ai", "tools"]
categories = ["command-line-utilities", "text-processing"]

[[bin]]
name = "scrivener-mcp"
path = "src/main.rs"

[dependencies]
rmcp = { version = "0.1", features = ["server", "transport-io", "macros"] }
scrivener = "0.1"
writing-analysis = "0.1"
rusqlite = { version = "0.32", features = ["bundled"] }
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
schemars = "0.8"
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"
uuid = { version = "1.12", features = ["v4", "serde"] }

[dev-dependencies]
anyhow = "1.0"
tempfile = "3.8"
pretty_assertions = "1.4"
```

## Implementation Order

Each step builds on the previous. Run `cargo check` after each step.

### Step 1: Project Scaffolding (~30 min)

- Create `Cargo.toml` as above
- Create `src/main.rs` with a minimal `#[tokio::main]` that prints "scrivener-mcp starting"
- Create module stubs: `src/server.rs`, `src/types.rs`, `src/error.rs`
- Create handler stubs: `src/handlers/mod.rs`, `project.rs`, `document.rs`, `search.rs`, `compile.rs`, `analysis.rs`, `memory.rs`
- Create service stubs: `src/services/mod.rs`, `project.rs`, `database.rs`, `analysis.rs`
- Run `cargo check` to verify compilation

**Expected output**: Project compiles with empty modules.

### Step 2: Error Types (~30 min)

- Implement `error.rs` with `McpError` enum wrapping upstream errors
- Add `From` impls: `scrivener::ScrivenerError`, `writing_analysis::WritingAnalysisError`, `rusqlite::Error`, `std::io::Error`
- Add helper `fn to_mcp_error() -> rmcp::ErrorData` conversion

**Expected output**: `cargo check` passes.

### Step 3: Core Types (~1 hour)

- Implement `types.rs`:
  - Tool input params: `OpenProjectParams`, `ReadDocumentParams`, `SearchContentParams`, etc.
  - Tool output types: `ProjectInfo`, `DocumentInfo`, `SearchResultItem`, etc.
  - All types derive `Serialize, Deserialize, JsonSchema`
- Implement `services/project.rs`: `ProjectSession` struct

**Expected output**: All types compile. `cargo check` passes.

### Step 4: Server Bootstrap (~1 day)

- Implement `server.rs`:
  - `ScrivenerMcp` struct with shared state (`Arc<Mutex<...>>`)
  - `impl ServerHandler for ScrivenerMcp` with `get_info()` returning `ServerInfo`
  - Wire up `#[tool_handler]` macro for `call_tool` and `list_tools` delegation
- Implement `main.rs`:
  - `clap` CLI args (log level, db path)
  - `tracing_subscriber` init (stderr output)
  - Create `ScrivenerMcp` instance
  - Start server: `server.serve(rmcp::transport::io::stdio()).await`
- Verify: run binary, confirm it starts and responds to MCP `initialize` request

**Expected output**: Server starts, responds to `initialize`, returns `ServerInfo` with name and version.

### Step 5: Project Handlers (~1 day)

- Implement `handlers/project.rs` with `#[tool_router]`:
  - `open_project`: validate path → `scrivener::Project::open()` → store in `ProjectSession`
  - `close_project`: clear `ProjectSession`
  - `refresh_project`: close + re-open from same path
  - `get_structure`: read binder tree → format as hierarchical JSON
- Wire into `ScrivenerMcp` tool router composition

**Expected output**: Can open a real `.scriv` project via MCP tool call and get its structure.

### Step 6: Document Handlers (~2 days)

- Implement `handlers/document.rs` with `#[tool_router]`:
  - `read_document`: find by UUID/title → `Document::read_content()` → return text
  - `write_document`: find document → `Document::write_content()` → save
  - `create_document`: create new document in specified folder → save binder
  - `delete_document`: move to trash → save binder
  - `rename_document`: update title → save binder
  - `move_document`: `Binder::move_item()` → save binder
  - `get_document_info`: return metadata, word count, keywords, synopsis
  - `update_metadata`: update synopsis, notes, keywords, label, status

**Expected output**: Full document CRUD via MCP tools. Round-trip: create → write → read → verify.

### Step 7: Search Handlers (~1 day)

- Implement `handlers/search.rs` with `#[tool_router]`:
  - `search_content`: `Project::search()` → format results with context
  - `list_trash`: `Project::list_trash()` → return trash items
  - `search_trash`: filter trash items by query
  - `recover_document`: `Project::recover_from_trash()` → save

**Expected output**: Search across project documents, trash management.

### Step 8: Compilation Handlers (~1 day)

- Implement `handlers/compile.rs` with `#[tool_router]`:
  - `compile_documents`: collect documents in reading order → concatenate content
  - `export_project`: compile to single text/markdown output
  - `get_statistics`: `Project::statistics()` → format as JSON

**Expected output**: Compile draft to single output, get project-wide statistics.

### Step 9: Analysis Handlers (~1 day)

- Implement `handlers/analysis.rs` with `#[tool_router]`:
  - `analyze_document`: read content → `writing_analysis::analyze_all()` → format results
  - `get_word_count`: read content → count words/characters/sentences
  - `analyze_readability`: read content → `writing_analysis::analyze_readability()` → format scores
- Implement `services/analysis.rs`: thin wrapper calling `writing_analysis` functions

**Expected output**: Full text analysis on any document via MCP tools.

### Step 10: SQLite Integration (~1 day)

- Implement `services/database.rs`:
  - `Database::open(path)` → create tables if not exist
  - Tables: `project_memory`, `analysis_cache`, `session_history`
  - CRUD functions for each table
- Schema:

```sql
CREATE TABLE IF NOT EXISTS project_memory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_path TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,
    category TEXT DEFAULT 'general',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(project_path, key)
);

CREATE TABLE IF NOT EXISTS analysis_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_uuid TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    analysis_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(document_uuid, content_hash)
);

CREATE TABLE IF NOT EXISTS session_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_path TEXT NOT NULL,
    action TEXT NOT NULL,
    details TEXT,
    timestamp TEXT NOT NULL
);
```

**Expected output**: Database creates tables, stores and retrieves data.

### Step 11: Memory Handlers (~1 day)

- Implement `handlers/memory.rs` with `#[tool_router]`:
  - `update_memory`: store key-value pair for current project
  - `get_memory`: retrieve memory entries by key or category
  - `check_consistency`: compare stored character/plot notes against current project state
  - `get_project_summary`: aggregate memory + statistics into a summary
- Wire into `ScrivenerMcp`

**Expected output**: Persistent project memory across sessions.

### Step 12: Polish (~1 day)

- CLI args: `--db-path`, `--log-level`, `--version`
- Improve error messages with context
- Add `get_writing_stats` and `get_session_info` tools
- Verify all 28 tools work end-to-end
- `cargo clippy` clean, `cargo test` passes
- README with installation and configuration instructions

**Expected output**: Production-ready binary. All tools functional.

## Known Challenges & Solutions

### 1. Tool Router Composition

**Problem**: rmcp's `#[tool_router]` generates a `ToolRouter<Self>` — composing routers from different structs requires type erasure or a unified dispatch.

**Solution**: Use the `+` operator on `ToolRouter` instances to merge them, or implement a manual `call_tool` dispatcher in `ServerHandler` that tries each handler in sequence. If `+` composition isn't supported, use a `HashMap<String, Box<dyn ToolHandler>>` for manual dispatch.

### 2. Shared State Across Handlers

**Problem**: Multiple handler groups need access to the same `ProjectSession` and `Database`.

**Solution**: Each handler struct holds `Arc<Mutex<Option<ProjectSession>>>` and `Arc<Database>` cloned from the main `ScrivenerMcp`. The `Arc` allows shared ownership; the `Mutex` ensures exclusive access during mutations.

### 3. Blocking I/O in Async Context

**Problem**: `scrivener` crate uses synchronous file I/O. Calling `Project::open()` or `Document::read_content()` blocks the tokio runtime.

**Solution**: Wrap blocking calls in `tokio::task::spawn_blocking()`. The state `Arc<Mutex<...>>` is `Send + Sync`, so it can cross the spawn_blocking boundary.

```rust
let project = tokio::task::spawn_blocking(move || {
    scrivener::Project::open(&path)
}).await??;
```

### 4. SQLite in Async Context

**Problem**: `rusqlite::Connection` is not `Send`, cannot be shared across async tasks directly.

**Solution**: Use `tokio::task::spawn_blocking()` for all database operations, or use a dedicated database thread with a channel-based API. For simplicity, wrap the `Connection` in a `Mutex` and use `spawn_blocking` for queries.

### 5. Large Document Content

**Problem**: Some Scrivener documents may contain very large RTF content. Returning the full text in a single MCP tool response could be slow.

**Solution**: Add an optional `max_length` parameter to `read_document`. If set, truncate the response and include a `truncated: true` flag. For analysis tools, process the full content but summarize results.

### 6. Project State Consistency

**Problem**: If Scrivener modifies the project while the MCP server has it open, the in-memory state becomes stale.

**Solution**: `refresh_project` tool reloads from disk. Document read operations always read from disk (lazy loading), so content is always fresh. The binder structure is cached but can be refreshed.

### 7. Error Recovery

**Problem**: If a tool call fails (e.g., document not found), the server should continue running — not crash.

**Solution**: All handler functions return `Result<CallToolResult, McpError>`. Errors are mapped to MCP error responses, not panics. The `#[tool_router]` macro handles this automatically. Use `?` with the error mapping conversion.
