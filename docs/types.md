# Complete Type Definitions

## Error Types (`error.rs`)

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("No project is currently open")]
    NoProjectOpen,

    #[error("Project already open: {path}")]
    ProjectAlreadyOpen {
        path: String,
    },

    #[error("Document not found: {identifier}")]
    DocumentNotFound {
        identifier: String,
    },

    #[error("Invalid parameter: {message}")]
    InvalidParameter {
        message: String,
    },

    #[error("Scrivener error: {0}")]
    Scrivener(#[from] scrivener::ScrivenerError),

    #[error("Analysis error: {0}")]
    Analysis(#[from] writing_analysis::WritingAnalysisError),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl From<McpError> for rmcp::ErrorData {
    fn from(e: McpError) -> Self {
        rmcp::ErrorData {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: e.to_string().into(),
            data: None,
        }
    }
}
```

### Error Notes

- `NoProjectOpen` is returned by any tool that requires an open project (most tools except `open_project`)
- `ProjectAlreadyOpen` is returned when `open_project` is called while a project is already open
- `DocumentNotFound` includes the identifier used (UUID or title) for debugging
- All upstream errors (`scrivener`, `writing_analysis`, `rusqlite`) are wrapped via `#[from]`
- The `From<McpError> for rmcp::ErrorData` impl converts to MCP-compatible error responses

## Server State (`server.rs`)

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

/// Main MCP server struct, holds shared state and tool routers.
#[derive(Clone)]
pub struct ScrivenerMcp {
    /// Currently open project session (None if no project is open).
    pub session: Arc<Mutex<Option<ProjectSession>>>,

    /// SQLite database for persistence.
    pub database: Arc<Database>,

    /// Combined tool router for dispatching tool calls.
    pub tool_router: ToolRouter<Self>,
}
```

## Project Session (`services/project.rs`)

```rust
/// Represents an open Scrivener project session.
///
/// Created by `open_project`, destroyed by `close_project`.
/// Only one session can be active at a time.
pub struct ProjectSession {
    /// The loaded Scrivener project.
    pub project: scrivener::Project,

    /// Absolute path to the .scriv bundle.
    pub project_path: PathBuf,

    /// When this session was started.
    pub opened_at: chrono::DateTime<chrono::Utc>,
}
```

## Tool Input Types (`types.rs`)

All input types derive `Deserialize` and `JsonSchema` for automatic parameter validation.

### Project Tools

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for open_project tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenProjectParams {
    /// Absolute path to the .scriv bundle directory.
    pub path: String,
}

/// Parameters for refresh_project tool (no params needed, uses current session).
/// close_project also uses no params.
```

### Document Tools

```rust
/// Parameters for read_document tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadDocumentParams {
    /// Document UUID or title to look up.
    pub identifier: String,

    /// Maximum content length to return (optional, returns full content if omitted).
    #[serde(default)]
    pub max_length: Option<usize>,
}

/// Parameters for write_document tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteDocumentParams {
    /// Document UUID or title to look up.
    pub identifier: String,

    /// New content to write (plain text, will be converted to RTF).
    pub content: String,
}

/// Parameters for create_document tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateDocumentParams {
    /// Title for the new document.
    pub title: String,

    /// UUID of the parent folder (optional, defaults to Draft folder).
    #[serde(default)]
    pub parent_uuid: Option<String>,

    /// Initial content (optional).
    #[serde(default)]
    pub content: Option<String>,
}

/// Parameters for create_folder tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFolderParams {
    /// Title for the new folder.
    pub title: String,

    /// UUID of the parent folder (optional, defaults to Draft folder).
    #[serde(default)]
    pub parent_uuid: Option<String>,
}

/// Parameters for delete_document tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDocumentParams {
    /// Document UUID or title to delete (moves to trash).
    pub identifier: String,
}

/// Parameters for rename_document tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameDocumentParams {
    /// Document UUID or title to rename.
    pub identifier: String,

    /// New title.
    pub new_title: String,
}

/// Parameters for move_document tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveDocumentParams {
    /// Document UUID to move.
    pub uuid: String,

    /// UUID of the target parent folder (None = move to root).
    #[serde(default)]
    pub target_parent_uuid: Option<String>,
}

/// Parameters for get_document_info tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDocumentInfoParams {
    /// Document UUID or title to look up.
    pub identifier: String,
}

/// Parameters for update_metadata tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateMetadataParams {
    /// Document UUID or title to update.
    pub identifier: String,

    /// New synopsis text (optional).
    #[serde(default)]
    pub synopsis: Option<String>,

    /// New notes text (optional).
    #[serde(default)]
    pub notes: Option<String>,

    /// Keywords to add (optional).
    #[serde(default)]
    pub add_keywords: Option<Vec<String>>,

    /// Keywords to remove (optional).
    #[serde(default)]
    pub remove_keywords: Option<Vec<String>>,
}
```

### Search Tools

```rust
/// Parameters for search_content tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchContentParams {
    /// Search query string.
    pub query: String,

    /// Use regex matching (optional, defaults to false).
    #[serde(default)]
    pub regex: bool,

    /// Maximum number of results (optional, defaults to 50).
    #[serde(default)]
    pub max_results: Option<usize>,
}

/// Parameters for search_trash tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchTrashParams {
    /// Search query string.
    pub query: String,
}

/// Parameters for recover_document tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecoverDocumentParams {
    /// UUID of the trashed document to recover.
    pub uuid: String,
}
```

### Compilation Tools

```rust
/// Parameters for compile_documents tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompileDocumentsParams {
    /// UUID of the folder to compile (optional, defaults to Draft folder).
    #[serde(default)]
    pub folder_uuid: Option<String>,

    /// Output format: "text" or "markdown" (optional, defaults to "text").
    #[serde(default)]
    pub format: Option<String>,

    /// Include only documents marked for compile (optional, defaults to true).
    #[serde(default)]
    pub compile_only: Option<bool>,
}

/// Parameters for export_project tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportProjectParams {
    /// Output format: "text", "markdown" (optional, defaults to "text").
    #[serde(default)]
    pub format: Option<String>,
}
```

### Analysis Tools

```rust
/// Parameters for analyze_document tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeDocumentParams {
    /// Document UUID or title to analyze.
    pub identifier: String,

    /// Analysis types to run (optional, defaults to all).
    /// Possible values: "readability", "passive_voice", "cliches", "filter_words", "sentiment", "sentence_variety"
    #[serde(default)]
    pub analyses: Option<Vec<String>>,
}

/// Parameters for get_word_count tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetWordCountParams {
    /// Document UUID or title (optional, returns project-wide count if omitted).
    #[serde(default)]
    pub identifier: Option<String>,
}

/// Parameters for analyze_readability tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeReadabilityParams {
    /// Document UUID or title to analyze.
    pub identifier: String,
}
```

### Memory Tools

```rust
/// Parameters for update_memory tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateMemoryParams {
    /// Memory key (e.g., "protagonist_profile", "plot_outline").
    pub key: String,

    /// Memory value (free-form text).
    pub value: String,

    /// Category for organizing memories (optional, defaults to "general").
    /// Suggested: "character", "plot", "setting", "theme", "notes"
    #[serde(default)]
    pub category: Option<String>,
}

/// Parameters for get_memory tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMemoryParams {
    /// Memory key to retrieve (optional, returns all if omitted).
    #[serde(default)]
    pub key: Option<String>,

    /// Filter by category (optional).
    #[serde(default)]
    pub category: Option<String>,
}

/// Parameters for check_consistency tool.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheckConsistencyParams {
    /// Aspects to check (optional, defaults to all).
    /// Possible values: "characters", "timeline", "locations", "plot"
    #[serde(default)]
    pub aspects: Option<Vec<String>>,
}
```

## Tool Output Types (`types.rs`)

All output types derive `Serialize` for JSON responses.

```rust
/// Project structure information.
#[derive(Debug, Serialize)]
pub struct ProjectInfo {
    pub title: String,
    pub author: Option<String>,
    pub path: String,
    pub document_count: usize,
    pub folder_count: usize,
    pub total_words: usize,
}

/// Binder item in the project structure tree.
#[derive(Debug, Serialize)]
pub struct BinderItemInfo {
    pub uuid: String,
    pub title: String,
    pub item_type: String,  // "document" or "folder"
    pub children: Vec<BinderItemInfo>,
    pub include_in_compile: bool,
}

/// Document detailed information.
#[derive(Debug, Serialize)]
pub struct DocumentInfo {
    pub uuid: String,
    pub title: String,
    pub synopsis: Option<String>,
    pub keywords: Vec<String>,
    pub word_count: usize,
    pub character_count: usize,
    pub created: String,
    pub modified: String,
    pub include_in_compile: bool,
    pub path: Vec<String>,  // breadcrumb path in binder
}

/// Search result item.
#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub document_uuid: String,
    pub document_title: String,
    pub matches: Vec<SearchMatch>,
}

/// Individual search match.
#[derive(Debug, Serialize)]
pub struct SearchMatch {
    pub context: String,
    pub offset: usize,
}

/// Memory entry.
#[derive(Debug, Serialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Project summary combining memory and statistics.
#[derive(Debug, Serialize)]
pub struct ProjectSummary {
    pub project_info: ProjectInfo,
    pub memory_entries: Vec<MemoryEntry>,
    pub recent_sessions: Vec<SessionEntry>,
}

/// Session history entry.
#[derive(Debug, Serialize)]
pub struct SessionEntry {
    pub project_path: String,
    pub action: String,
    pub details: Option<String>,
    pub timestamp: String,
}

/// Writing statistics (from get_writing_stats tool).
#[derive(Debug, Serialize)]
pub struct WritingStats {
    pub total_words: usize,
    pub total_characters: usize,
    pub total_documents: usize,
    pub total_folders: usize,
    pub words_by_document: Vec<DocumentWordCount>,
}

/// Per-document word count.
#[derive(Debug, Serialize)]
pub struct DocumentWordCount {
    pub uuid: String,
    pub title: String,
    pub word_count: usize,
}
```

## Database Types (`services/database.rs`)

```rust
/// SQLite database wrapper for project memory and caching.
pub struct Database {
    /// Path to the SQLite database file.
    pub path: PathBuf,

    /// Connection (wrapped for async safety).
    conn: Mutex<rusqlite::Connection>,
}
```

## Key Design Choices

### 1. `String` for UUIDs in Tool Types

Tool input/output types use `String` for UUIDs (not `uuid::Uuid`) because JSON Schema represents them as strings. Parsing to `Uuid` happens inside handlers, with validation errors returned as `McpError::InvalidParameter`.

### 2. `Option<T>` with `#[serde(default)]` for Optional Params

Optional tool parameters use `Option<T>` with `#[serde(default)]`. This means MCP clients can omit these fields entirely. Default values are applied in handler logic, not in the type definitions.

### 3. Output as `Content::text` with JSON

Tool results are serialized to JSON strings and returned as `Content::text(json_string)`. This is the most compatible approach — all MCP clients can display text content. For structured output support (rmcp `Json<T>` wrapper), the output types also derive `JsonSchema`.

### 4. `tokio::sync::Mutex` for State

`tokio::sync::Mutex` (not `std::sync::Mutex`) is used for `ProjectSession` because the lock may be held across `.await` points (e.g., during `spawn_blocking` calls). For the database `Connection`, `std::sync::Mutex` is sufficient since it's only held during synchronous `spawn_blocking` calls.
