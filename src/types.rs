use std::collections::HashMap;

use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};

// ── Project Tool Params ─────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct OpenProjectParams {
    /// Absolute path to the .scriv bundle directory.
    pub path: String,
}

// ── Document Tool Params ────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadDocumentParams {
    /// Document UUID or title.
    pub identifier: String,

    /// Maximum content length to return.
    #[serde(default)]
    pub max_length: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WriteDocumentParams {
    /// Document UUID or title.
    pub identifier: String,

    /// New content to write (plain text, converted to RTF).
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateDocumentParams {
    /// Title for the new document.
    pub title: String,

    /// UUID of the parent folder (defaults to Draft folder).
    #[serde(default)]
    pub parent_uuid: Option<String>,

    /// Initial content.
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateFolderParams {
    /// Title for the new folder.
    pub title: String,

    /// UUID of the parent folder (defaults to Draft folder).
    #[serde(default)]
    pub parent_uuid: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteDocumentParams {
    /// Document UUID or title to delete (moves to trash).
    pub identifier: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameDocumentParams {
    /// Document UUID or title.
    pub identifier: String,

    /// New title.
    pub new_title: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MoveDocumentParams {
    /// UUID of the item to move.
    pub uuid: String,

    /// UUID of the target parent folder (omit to move to root).
    #[serde(default)]
    pub target_parent_uuid: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDocumentInfoParams {
    /// Document UUID or title.
    pub identifier: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateMetadataParams {
    /// Document UUID or title.
    pub identifier: String,

    /// New synopsis text.
    #[serde(default)]
    pub synopsis: Option<String>,

    /// New notes text.
    #[serde(default)]
    pub notes: Option<String>,

    /// Keywords to add.
    #[serde(default)]
    pub add_keywords: Option<Vec<String>>,

    /// Keywords to remove.
    #[serde(default)]
    pub remove_keywords: Option<Vec<String>>,

    /// Custom metadata key-value pairs to set or update.
    #[serde(default)]
    pub custom_metadata: Option<HashMap<String, String>>,
}

// ── Search Tool Params ──────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchContentParams {
    /// Search query string.
    pub query: String,

    /// Use regex matching.
    #[serde(default)]
    pub regex: bool,

    /// Maximum number of results.
    #[serde(default)]
    pub max_results: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchTrashParams {
    /// Search query string.
    pub query: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RecoverDocumentParams {
    /// UUID of the trashed document to recover.
    pub uuid: String,
}

// ── Compilation Tool Params ─────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CompileDocumentsParams {
    /// UUID of the folder to compile (defaults to Draft folder).
    #[serde(default)]
    pub folder_uuid: Option<String>,

    /// Output format: "text" or "markdown".
    #[serde(default)]
    pub format: Option<String>,

    /// Include only documents marked for compile.
    #[serde(default)]
    pub compile_only: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExportProjectParams {
    /// Output format: "text" or "markdown".
    #[serde(default)]
    pub format: Option<String>,
}

// ── Analysis Tool Params ────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeDocumentParams {
    /// Document UUID or title.
    pub identifier: String,

    /// Analysis types to run (defaults to all).
    /// Values: "readability", "passive_voice", "cliches", "filter_words", "sentiment", "sentence_variety"
    #[serde(default)]
    pub analyses: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetWordCountParams {
    /// Document UUID or title (omit for project-wide count).
    #[serde(default)]
    pub identifier: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct AnalyzeReadabilityParams {
    /// Document UUID or title.
    pub identifier: String,
}

// ── Memory Tool Params ──────────────────────────────────────────

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateMemoryParams {
    /// Memory key (e.g., "protagonist_profile", "plot_outline").
    pub key: String,

    /// Memory value (free-form text).
    pub value: String,

    /// Category: character, plot, setting, theme, notes (default: general).
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMemoryParams {
    /// Specific key to retrieve (omit for all).
    #[serde(default)]
    pub key: Option<String>,

    /// Filter by category.
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CheckConsistencyParams {
    /// Aspects to check: "characters", "timeline", "locations", "plot".
    #[serde(default)]
    #[allow(dead_code)]
    pub aspects: Option<Vec<String>>,
}

// ── Output Types ────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct ProjectInfo {
    pub title: String,
    pub author: Option<String>,
    pub path: String,
    pub document_count: usize,
    pub folder_count: usize,
    pub total_words: usize,
}

#[derive(Debug, Serialize)]
pub struct BinderItemInfo {
    pub uuid: String,
    pub title: String,
    pub item_type: String,
    pub children: Vec<BinderItemInfo>,
    pub include_in_compile: bool,
}

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
    pub path: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub document_uuid: String,
    pub document_title: String,
    pub matches: Vec<SearchMatchItem>,
}

#[derive(Debug, Serialize)]
pub struct SearchMatchItem {
    pub context: String,
    pub offset: usize,
}

#[derive(Debug, Serialize)]
pub struct MemoryEntry {
    pub key: String,
    pub value: String,
    pub category: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct ProjectSummary {
    pub project_info: ProjectInfo,
    pub memory_entries: Vec<MemoryEntry>,
}

#[derive(Debug, Serialize)]
pub struct WritingStats {
    pub total_words: usize,
    pub total_characters: usize,
    pub total_documents: usize,
    pub total_folders: usize,
    pub words_by_document: Vec<DocumentWordCount>,
}

#[derive(Debug, Serialize)]
pub struct DocumentWordCount {
    pub uuid: String,
    pub title: String,
    pub word_count: usize,
}

#[derive(Debug, Serialize)]
pub struct SessionInfo {
    pub server_version: String,
    pub current_project: Option<String>,
    pub session_start: String,
}
