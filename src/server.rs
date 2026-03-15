use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::*,
    tool, tool_handler, tool_router, ServerHandler,
};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::error::McpServerError;
use crate::services::database::Database;
use crate::services::project::ProjectSession;
use crate::types::*;

#[derive(Clone)]
pub struct ScrivenerMcp {
    session: Arc<Mutex<Option<ProjectSession>>>,
    database: Arc<Database>,
    tool_router: ToolRouter<Self>,
    start_time: chrono::DateTime<chrono::Utc>,
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for ScrivenerMcp {
    fn get_info(&self) -> ServerInfo {
        let mut caps = ServerCapabilities::default();
        caps.tools = Some(ToolsCapability::default());
        ServerInfo::new(caps)
            .with_server_info(Implementation::new(
                "scrivener-mcp",
                env!("CARGO_PKG_VERSION"),
            ))
            .with_instructions(
                "Scrivener 3 project MCP server. Use open_project to start, then access documents, search, analyze, and manage project memory.",
            )
    }
}

impl ScrivenerMcp {
    pub fn new(database: Database) -> Self {
        let session = Arc::new(Mutex::new(None));
        let db = Arc::new(database);

        let tool_router = Self::project_router()
            + Self::document_router()
            + Self::search_router()
            + Self::compile_router()
            + Self::analysis_router()
            + Self::memory_router()
            + Self::stats_router();

        Self {
            session,
            database: db,
            tool_router,
            start_time: chrono::Utc::now(),
        }
    }
}

// ── Helper ──────────────────────────────────────────────────────

fn find_doc_uuid(binder: &scrivener::Binder, identifier: &str) -> Result<Uuid, McpServerError> {
    if let Ok(uuid) = Uuid::parse_str(identifier) {
        if binder.find_by_uuid(uuid).is_some() {
            return Ok(uuid);
        }
    }
    let matches = binder.find_by_title(identifier);
    match matches.len() {
        0 => Err(McpServerError::DocumentNotFound {
            identifier: identifier.to_string(),
        }),
        _ => {
            for m in &matches {
                if m.title() == identifier {
                    return Ok(m.uuid());
                }
            }
            Ok(matches[0].uuid())
        }
    }
}

fn binder_item_to_info(item: &scrivener::BinderItem) -> BinderItemInfo {
    match item {
        scrivener::BinderItem::Document(doc) => BinderItemInfo {
            uuid: doc.uuid.to_string(),
            title: doc.title.clone(),
            item_type: "document".to_string(),
            children: vec![],
            include_in_compile: doc.metadata.include_in_compile,
        },
        scrivener::BinderItem::Folder(folder) => BinderItemInfo {
            uuid: folder.uuid.to_string(),
            title: folder.title.clone(),
            item_type: "folder".to_string(),
            children: folder.children.iter().map(binder_item_to_info).collect(),
            include_in_compile: folder.metadata.include_in_compile,
        },
    }
}

fn remove_from_binder(
    items: &mut Vec<scrivener::BinderItem>,
    uuid: Uuid,
) -> Option<scrivener::BinderItem> {
    if let Some(pos) = items.iter().position(|i| i.uuid() == uuid) {
        return Some(items.remove(pos));
    }
    for item in items.iter_mut() {
        if let scrivener::BinderItem::Folder(folder) = item {
            if let Some(removed) = remove_from_binder(&mut folder.children, uuid) {
                return Some(removed);
            }
        }
    }
    None
}

fn compile_items(
    items: &[scrivener::BinderItem],
    project_path: &std::path::Path,
    compile_only: bool,
    format: &str,
) -> String {
    let mut output = String::new();
    for item in items {
        match item {
            scrivener::BinderItem::Document(doc) => {
                if compile_only && !doc.metadata.include_in_compile {
                    continue;
                }
                if let Ok(content) = doc.read_content(project_path) {
                    if let Some(text) = &content.plain_text {
                        if !text.is_empty() {
                            if format == "markdown" {
                                output.push_str(&format!("## {}\n\n", doc.title));
                            } else {
                                output.push_str(&format!("--- {} ---\n\n", doc.title));
                            }
                            output.push_str(text);
                            output.push_str("\n\n");
                        }
                    }
                }
            }
            scrivener::BinderItem::Folder(folder) => {
                if format == "markdown" {
                    output.push_str(&format!("## {}\n\n", folder.title));
                }
                output.push_str(&compile_items(
                    &folder.children,
                    project_path,
                    compile_only,
                    format,
                ));
            }
        }
    }
    output
}

fn build_analysis(text: &str, analyses: Option<&[String]>) -> String {
    let run_all = analyses.is_none();
    let should_run =
        |name: &str| -> bool { run_all || analyses.unwrap().iter().any(|a| a == name) };

    let mut result = serde_json::Map::new();

    if should_run("readability") {
        if let Ok(scores) = writing_analysis::analyze_readability(text) {
            result.insert(
                "readability".into(),
                serde_json::json!({
                    "flesch_kincaid_grade": scores.flesch_kincaid_grade,
                    "flesch_reading_ease": scores.flesch_reading_ease,
                    "smog_index": scores.smog_index,
                    "coleman_liau_index": scores.coleman_liau_index,
                    "automated_readability_index": scores.automated_readability_index,
                }),
            );
        }
    }
    if should_run("passive_voice") {
        if let Ok(pv) = writing_analysis::detect_passive_voice(text) {
            result.insert("passive_voice".into(), serde_json::json!({
                "percentage": pv.percentage,
                "instance_count": pv.instances.len(),
                "instances": pv.instances.iter().map(|i| serde_json::json!({"phrase": i.phrase, "sentence": i.sentence})).collect::<Vec<_>>(),
            }));
        }
    }
    if should_run("cliches") {
        if let Ok(cl) = writing_analysis::detect_cliches(text) {
            result.insert("cliches".into(), serde_json::json!({
                "count": cl.count,
                "instances": cl.instances.iter().map(|i| serde_json::json!({"phrase": i.phrase, "canonical": i.canonical})).collect::<Vec<_>>(),
            }));
        }
    }
    if should_run("filter_words") {
        if let Ok(fw) = writing_analysis::detect_filter_words(text) {
            result.insert(
                "filter_words".into(),
                serde_json::json!({
                    "count": fw.count,
                    "percentage": fw.percentage,
                }),
            );
        }
    }
    if should_run("sentiment") {
        if let Ok(s) = writing_analysis::analyze_sentiment(text) {
            result.insert(
                "sentiment".into(),
                serde_json::json!({
                    "score": s.score,
                    "comparative": s.comparative,
                }),
            );
        }
    }
    if should_run("sentence_variety") {
        if let Ok(sv) = writing_analysis::analyze_sentence_variety(text) {
            result.insert(
                "sentence_variety".into(),
                serde_json::json!({
                    "avg_length": sv.avg_length,
                    "length_variance": sv.length_variance,
                    "structure_variety": sv.structure_variety,
                }),
            );
        }
    }

    serde_json::to_string_pretty(&serde_json::Value::Object(result)).unwrap_or_default()
}

// ── Project Tools ───────────────────────────────────────────────

#[tool_router(router = project_router)]
impl ScrivenerMcp {
    #[tool(
        name = "open_project",
        description = "Open a Scrivener 3 project (.scriv bundle) from the specified path"
    )]
    async fn open_project(&self, Parameters(params): Parameters<OpenProjectParams>) -> String {
        match self.do_open_project(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "close_project",
        description = "Close the currently open Scrivener project"
    )]
    async fn close_project(&self) -> String {
        let mut session = self.session.lock().await;
        match session.take() {
            Some(s) => {
                let _ =
                    self.database
                        .log_session(&s.project_path.display().to_string(), "close", None);
                format!("Project closed: {}", s.project_path.display())
            }
            None => "No project is currently open".to_string(),
        }
    }

    #[tool(
        name = "refresh_project",
        description = "Reload the current project from disk to pick up external changes"
    )]
    async fn refresh_project(&self) -> String {
        match self.do_refresh_project().await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "get_structure",
        description = "Get the hierarchical binder structure showing all documents and folders"
    )]
    async fn get_structure(&self) -> String {
        let session = self.session.lock().await;
        match session.as_ref() {
            None => "Error: No project is currently open".to_string(),
            Some(s) => {
                let items: Vec<BinderItemInfo> = s
                    .project
                    .binder
                    .root
                    .iter()
                    .map(binder_item_to_info)
                    .collect();
                serde_json::to_string_pretty(&items).unwrap_or_default()
            }
        }
    }
}

// ── Document Tools ──────────────────────────────────────────────

#[tool_router(router = document_router)]
impl ScrivenerMcp {
    #[tool(
        name = "read_document",
        description = "Read the text content of a document by UUID or title"
    )]
    async fn read_document(&self, Parameters(params): Parameters<ReadDocumentParams>) -> String {
        match self.do_read_document(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "write_document",
        description = "Write new text content to a document (replaces existing content)"
    )]
    async fn write_document(&self, Parameters(params): Parameters<WriteDocumentParams>) -> String {
        match self.do_write_document(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "create_document",
        description = "Create a new document in the specified folder (defaults to Draft)"
    )]
    async fn create_document(
        &self,
        Parameters(params): Parameters<CreateDocumentParams>,
    ) -> String {
        match self.do_create_document(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "delete_document",
        description = "Move a document to the trash (can be recovered later)"
    )]
    async fn delete_document(
        &self,
        Parameters(params): Parameters<DeleteDocumentParams>,
    ) -> String {
        match self.do_delete_document(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(name = "rename_document", description = "Rename a document or folder")]
    async fn rename_document(
        &self,
        Parameters(params): Parameters<RenameDocumentParams>,
    ) -> String {
        match self.do_rename_document(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "move_document",
        description = "Move a document or folder to a different parent folder"
    )]
    async fn move_document(&self, Parameters(params): Parameters<MoveDocumentParams>) -> String {
        match self.do_move_document(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "get_document_info",
        description = "Get detailed information about a document including metadata, word count, and binder path"
    )]
    async fn get_document_info(
        &self,
        Parameters(params): Parameters<GetDocumentInfoParams>,
    ) -> String {
        match self.do_get_document_info(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "update_metadata",
        description = "Update document metadata: synopsis, notes, and/or keywords"
    )]
    async fn update_metadata(
        &self,
        Parameters(params): Parameters<UpdateMetadataParams>,
    ) -> String {
        match self.do_update_metadata(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }
}

// ── Search Tools ────────────────────────────────────────────────

#[tool_router(router = search_router)]
impl ScrivenerMcp {
    #[tool(
        name = "search_content",
        description = "Search for text content across all documents in the project"
    )]
    async fn search_content(&self, Parameters(params): Parameters<SearchContentParams>) -> String {
        match self.do_search_content(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "list_trash",
        description = "List all documents currently in the trash"
    )]
    async fn list_trash(&self) -> String {
        let session = self.session.lock().await;
        match session.as_ref() {
            None => "Error: No project is currently open".to_string(),
            Some(s) => {
                let items: Vec<serde_json::Value> = s.project.trash.items.iter().map(|item| {
                    let (title, typ) = match item {
                        scrivener::TrashedItem::Document(d) => (&d.title, "document"),
                        scrivener::TrashedItem::Folder(f) => (&f.title, "folder"),
                    };
                    serde_json::json!({"uuid": item.uuid().to_string(), "title": title, "type": typ})
                }).collect();
                serde_json::to_string_pretty(&items).unwrap_or_default()
            }
        }
    }

    #[tool(
        name = "search_trash",
        description = "Search for documents in the trash by title"
    )]
    async fn search_trash(&self, Parameters(params): Parameters<SearchTrashParams>) -> String {
        let session = self.session.lock().await;
        match session.as_ref() {
            None => "Error: No project is currently open".to_string(),
            Some(s) => {
                let lower_query = params.query.to_lowercase();
                let items: Vec<serde_json::Value> = s.project.trash.items.iter().filter(|item| {
                    let title = match item {
                        scrivener::TrashedItem::Document(d) => &d.title,
                        scrivener::TrashedItem::Folder(f) => &f.title,
                    };
                    title.to_lowercase().contains(&lower_query)
                }).map(|item| {
                    let (title, typ) = match item {
                        scrivener::TrashedItem::Document(d) => (&d.title, "document"),
                        scrivener::TrashedItem::Folder(f) => (&f.title, "folder"),
                    };
                    serde_json::json!({"uuid": item.uuid().to_string(), "title": title, "type": typ})
                }).collect();
                serde_json::to_string_pretty(&items).unwrap_or_default()
            }
        }
    }

    #[tool(
        name = "recover_document",
        description = "Recover a document from the trash back into the project binder"
    )]
    async fn recover_document(
        &self,
        Parameters(params): Parameters<RecoverDocumentParams>,
    ) -> String {
        match self.do_recover_document(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }
}

// ── Compile Tools ───────────────────────────────────────────────

#[tool_router(router = compile_router)]
impl ScrivenerMcp {
    #[tool(
        name = "compile_documents",
        description = "Compile documents in reading order into a single text output"
    )]
    async fn compile_documents(
        &self,
        Parameters(params): Parameters<CompileDocumentsParams>,
    ) -> String {
        match self.do_compile_documents(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "export_project",
        description = "Export the entire project draft as a single document"
    )]
    async fn export_project(&self, Parameters(params): Parameters<ExportProjectParams>) -> String {
        match self.do_export_project(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "get_statistics",
        description = "Get project statistics: document count, word count, and per-document breakdown"
    )]
    async fn get_statistics(&self) -> String {
        match self.do_get_statistics().await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }
}

// ── Analysis Tools ──────────────────────────────────────────────

#[tool_router(router = analysis_router)]
impl ScrivenerMcp {
    #[tool(
        name = "analyze_document",
        description = "Analyze a document for readability, passive voice, clichés, filter words, sentiment, and sentence variety"
    )]
    async fn analyze_document(
        &self,
        Parameters(params): Parameters<AnalyzeDocumentParams>,
    ) -> String {
        match self.do_analyze_document(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "get_word_count",
        description = "Get word count for a specific document or the entire project"
    )]
    async fn get_word_count(&self, Parameters(params): Parameters<GetWordCountParams>) -> String {
        match self.do_get_word_count(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "analyze_readability",
        description = "Analyze readability of a document (Flesch-Kincaid, SMOG, Coleman-Liau, ARI)"
    )]
    async fn analyze_readability(
        &self,
        Parameters(params): Parameters<AnalyzeReadabilityParams>,
    ) -> String {
        match self.do_analyze_readability(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }
}

// ── Memory Tools ────────────────────────────────────────────────

#[tool_router(router = memory_router)]
impl ScrivenerMcp {
    #[tool(
        name = "update_memory",
        description = "Store or update a memory entry for the current project (persists across sessions)"
    )]
    async fn update_memory(&self, Parameters(params): Parameters<UpdateMemoryParams>) -> String {
        match self.do_update_memory(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "get_memory",
        description = "Retrieve stored memory entries for the current project"
    )]
    async fn get_memory(&self, Parameters(params): Parameters<GetMemoryParams>) -> String {
        match self.do_get_memory(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "check_consistency",
        description = "Check stored project memory against current document state for inconsistencies"
    )]
    async fn check_consistency(
        &self,
        Parameters(params): Parameters<CheckConsistencyParams>,
    ) -> String {
        match self.do_check_consistency(params).await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "get_project_summary",
        description = "Get a comprehensive project summary including info, statistics, and stored memories"
    )]
    async fn get_project_summary(&self) -> String {
        match self.do_get_project_summary().await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }
}

// ── Stats Tools ─────────────────────────────────────────────────

#[tool_router(router = stats_router)]
impl ScrivenerMcp {
    #[tool(
        name = "get_writing_stats",
        description = "Get detailed writing statistics with per-document word count breakdown"
    )]
    async fn get_writing_stats(&self) -> String {
        match self.do_get_statistics().await {
            Ok(s) => s,
            Err(e) => format!("Error: {}", e),
        }
    }

    #[tool(
        name = "get_session_info",
        description = "Get information about the current MCP server session"
    )]
    async fn get_session_info(&self) -> String {
        let session = self.session.lock().await;
        let current_project = session
            .as_ref()
            .map(|s| s.project_path.display().to_string());
        let info = SessionInfo {
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            current_project,
            session_start: self.start_time.to_rfc3339(),
        };
        serde_json::to_string_pretty(&info).unwrap_or_default()
    }
}

// ── Implementation details ──────────────────────────────────────

impl ScrivenerMcp {
    async fn do_open_project(&self, params: OpenProjectParams) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        if let Some(ref s) = *session {
            return Err(McpServerError::ProjectAlreadyOpen {
                path: s.project_path.display().to_string(),
            });
        }

        let path = std::path::PathBuf::from(&params.path);
        let project = tokio::task::spawn_blocking(move || scrivener::Project::open(&path))
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        let stats = {
            let p = project.clone();
            tokio::task::spawn_blocking(move || p.statistics())
                .await
                .map_err(|e| McpServerError::InvalidParameter {
                    message: e.to_string(),
                })?
        };

        let info = ProjectInfo {
            title: project.metadata.title.clone(),
            author: project.metadata.author.clone(),
            path: project.path.display().to_string(),
            document_count: stats.total_documents,
            folder_count: stats.total_folders,
            total_words: stats.total_words,
        };

        let _ = self.database.log_session(
            &project.path.display().to_string(),
            "open",
            Some(&info.title),
        );

        *session = Some(ProjectSession {
            project,
            project_path: std::path::PathBuf::from(&params.path),
            opened_at: chrono::Utc::now(),
        });

        Ok(serde_json::to_string_pretty(&info)?)
    }

    async fn do_refresh_project(&self) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let path = s.project_path.clone();

        let project = tokio::task::spawn_blocking(move || scrivener::Project::open(&path))
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        let pp = project.path.clone();
        *session = Some(ProjectSession {
            project,
            project_path: pp,
            opened_at: chrono::Utc::now(),
        });
        Ok("Project refreshed successfully".to_string())
    }

    async fn do_read_document(&self, params: ReadDocumentParams) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = find_doc_uuid(&s.project.binder, &params.identifier)?;

        let item = s
            .project
            .binder
            .find_by_uuid(uuid)
            .ok_or(McpServerError::DocumentNotFound {
                identifier: params.identifier.clone(),
            })?;

        if let scrivener::BinderItem::Document(doc) = item {
            let pp = s.project.path.clone();
            let dc = doc.clone();
            let content = tokio::task::spawn_blocking(move || dc.read_content(&pp))
                .await
                .map_err(|e| McpServerError::InvalidParameter {
                    message: e.to_string(),
                })?
                .map_err(McpServerError::Scrivener)?;

            let mut text = content.plain_text.unwrap_or_default();
            if let Some(max_len) = params.max_length {
                if text.len() > max_len {
                    text.truncate(max_len);
                    text.push_str("\n\n[truncated]");
                }
            }
            Ok(text)
        } else {
            Err(McpServerError::InvalidParameter {
                message: format!("'{}' is a folder", params.identifier),
            })
        }
    }

    async fn do_write_document(&self, params: WriteDocumentParams) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        let s = session.as_mut().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = find_doc_uuid(&s.project.binder, &params.identifier)?;
        let pp = s.project.path.clone();

        let item =
            s.project
                .binder
                .find_by_uuid_mut(uuid)
                .ok_or(McpServerError::DocumentNotFound {
                    identifier: params.identifier.clone(),
                })?;

        if let scrivener::BinderItem::Document(doc) = item {
            doc.write_content(&pp, &params.content)
                .map_err(McpServerError::Scrivener)?;
            let wc = params.content.split_whitespace().count();
            Ok(format!("Document updated. Word count: {}", wc))
        } else {
            Err(McpServerError::InvalidParameter {
                message: format!("'{}' is a folder", params.identifier),
            })
        }
    }

    async fn do_create_document(
        &self,
        params: CreateDocumentParams,
    ) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        let s = session.as_mut().ok_or(McpServerError::NoProjectOpen)?;

        let new_uuid = Uuid::new_v4();
        let mut doc = scrivener::Document {
            uuid: new_uuid,
            title: params.title.clone(),
            ..Default::default()
        };

        if let Some(content) = &params.content {
            doc.write_content(&s.project.path, content)
                .map_err(McpServerError::Scrivener)?;
        }

        let new_item = scrivener::BinderItem::Document(doc);

        if let Some(parent_uuid_str) = &params.parent_uuid {
            let parent_uuid = Uuid::parse_str(parent_uuid_str).map_err(McpServerError::Uuid)?;
            s.project.binder.root.push(new_item);
            s.project
                .binder
                .move_item(new_uuid, Some(parent_uuid))
                .map_err(McpServerError::Scrivener)?;
        } else {
            let first_folder = s.project.binder.root.iter().find_map(|item| {
                if let scrivener::BinderItem::Folder(f) = item {
                    Some(f.uuid)
                } else {
                    None
                }
            });
            s.project.binder.root.push(new_item);
            if let Some(fid) = first_folder {
                s.project
                    .binder
                    .move_item(new_uuid, Some(fid))
                    .map_err(McpServerError::Scrivener)?;
            }
        }

        let project = s.project.clone();
        tokio::task::spawn_blocking(move || project.save())
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        Ok(serde_json::to_string_pretty(
            &serde_json::json!({"uuid": new_uuid.to_string(), "title": params.title}),
        )?)
    }

    async fn do_delete_document(
        &self,
        params: DeleteDocumentParams,
    ) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        let s = session.as_mut().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = find_doc_uuid(&s.project.binder, &params.identifier)?;
        let title = s
            .project
            .binder
            .find_by_uuid(uuid)
            .map(|i| i.title().to_string())
            .unwrap_or_default();

        let removed = remove_from_binder(&mut s.project.binder.root, uuid).ok_or(
            McpServerError::DocumentNotFound {
                identifier: params.identifier,
            },
        )?;

        let trashed = match removed {
            scrivener::BinderItem::Document(d) => scrivener::TrashedItem::Document(d),
            scrivener::BinderItem::Folder(f) => scrivener::TrashedItem::Folder(f),
        };
        s.project.trash.items.push(trashed);

        let project = s.project.clone();
        tokio::task::spawn_blocking(move || project.save())
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        Ok(format!("Moved to trash: {}", title))
    }

    async fn do_rename_document(
        &self,
        params: RenameDocumentParams,
    ) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        let s = session.as_mut().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = find_doc_uuid(&s.project.binder, &params.identifier)?;

        let item =
            s.project
                .binder
                .find_by_uuid_mut(uuid)
                .ok_or(McpServerError::DocumentNotFound {
                    identifier: params.identifier.clone(),
                })?;

        let old_title = item.title().to_string();
        match item {
            scrivener::BinderItem::Document(doc) => doc.title = params.new_title.clone(),
            scrivener::BinderItem::Folder(folder) => folder.title = params.new_title.clone(),
        }

        let project = s.project.clone();
        tokio::task::spawn_blocking(move || project.save())
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        Ok(format!("Renamed '{}' → '{}'", old_title, params.new_title))
    }

    async fn do_move_document(&self, params: MoveDocumentParams) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        let s = session.as_mut().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = Uuid::parse_str(&params.uuid).map_err(McpServerError::Uuid)?;
        let target = params
            .target_parent_uuid
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(McpServerError::Uuid)?;

        s.project
            .binder
            .move_item(uuid, target)
            .map_err(McpServerError::Scrivener)?;

        let project = s.project.clone();
        tokio::task::spawn_blocking(move || project.save())
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        Ok(format!(
            "Moved {} to {}",
            params.uuid,
            params.target_parent_uuid.as_deref().unwrap_or("root")
        ))
    }

    async fn do_get_document_info(
        &self,
        params: GetDocumentInfoParams,
    ) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = find_doc_uuid(&s.project.binder, &params.identifier)?;

        let flat = s.project.binder.flatten();
        let (_item, path) = flat.iter().find(|(i, _)| i.uuid() == uuid).ok_or(
            McpServerError::DocumentNotFound {
                identifier: params.identifier.clone(),
            },
        )?;

        let item = s.project.binder.find_by_uuid(uuid).unwrap();
        if let scrivener::BinderItem::Document(doc) = item {
            // Get word count from content
            let pp = s.project.path.clone();
            let dc = doc.clone();
            let content = tokio::task::spawn_blocking(move || dc.read_content(&pp))
                .await
                .map_err(|e| McpServerError::InvalidParameter {
                    message: e.to_string(),
                })?
                .map_err(McpServerError::Scrivener)?;

            let (wc, cc) = content
                .formatted
                .as_ref()
                .map(|f| (f.word_count, f.character_count))
                .unwrap_or((0, 0));

            let info = DocumentInfo {
                uuid: doc.uuid.to_string(),
                title: doc.title.clone(),
                synopsis: doc.synopsis.clone(),
                keywords: doc.keywords.clone(),
                word_count: wc,
                character_count: cc,
                created: doc.metadata.created.to_rfc3339(),
                modified: doc.metadata.modified.to_rfc3339(),
                include_in_compile: doc.metadata.include_in_compile,
                path: path.clone(),
            };
            Ok(serde_json::to_string_pretty(&info)?)
        } else {
            let info =
                serde_json::json!({"uuid": uuid.to_string(), "item_type": "folder", "path": path});
            Ok(serde_json::to_string_pretty(&info)?)
        }
    }

    async fn do_update_metadata(
        &self,
        params: UpdateMetadataParams,
    ) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        let s = session.as_mut().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = find_doc_uuid(&s.project.binder, &params.identifier)?;
        let pp = s.project.path.clone();

        let item =
            s.project
                .binder
                .find_by_uuid_mut(uuid)
                .ok_or(McpServerError::DocumentNotFound {
                    identifier: params.identifier.clone(),
                })?;

        if let scrivener::BinderItem::Document(doc) = item {
            if let Some(synopsis) = &params.synopsis {
                doc.update_synopsis(&pp, synopsis)
                    .map_err(McpServerError::Scrivener)?;
            }
            if let Some(notes) = &params.notes {
                doc.update_notes(&pp, notes)
                    .map_err(McpServerError::Scrivener)?;
            }
            if let Some(kws) = &params.add_keywords {
                for kw in kws {
                    doc.add_keyword(kw);
                }
            }
            if let Some(kws) = &params.remove_keywords {
                for kw in kws {
                    doc.remove_keyword(kw);
                }
            }

            let project = s.project.clone();
            tokio::task::spawn_blocking(move || project.save())
                .await
                .map_err(|e| McpServerError::InvalidParameter {
                    message: e.to_string(),
                })?
                .map_err(McpServerError::Scrivener)?;

            Ok(format!("Metadata updated for: {}", params.identifier))
        } else {
            Err(McpServerError::InvalidParameter {
                message: format!("'{}' is a folder", params.identifier),
            })
        }
    }

    async fn do_search_content(&self, params: SearchContentParams) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let project = s.project.clone();
        let query = params.query.clone();
        let use_regex = params.regex;
        let max_results = params.max_results.unwrap_or(50);

        let results = tokio::task::spawn_blocking(move || {
            if use_regex {
                project.search_regex(&query)
            } else {
                Ok(project.search(&query))
            }
        })
        .await
        .map_err(|e| McpServerError::InvalidParameter {
            message: e.to_string(),
        })?
        .map_err(McpServerError::Scrivener)?;

        let items: Vec<SearchResultItem> = results
            .into_iter()
            .take(max_results)
            .map(|r| SearchResultItem {
                document_uuid: r.document_uuid.to_string(),
                document_title: r.document_title,
                matches: r
                    .matches
                    .into_iter()
                    .map(|m| SearchMatchItem {
                        context: m.context,
                        offset: m.position.0,
                    })
                    .collect(),
            })
            .collect();

        Ok(serde_json::to_string_pretty(&items)?)
    }

    async fn do_recover_document(
        &self,
        params: RecoverDocumentParams,
    ) -> crate::error::Result<String> {
        let mut session = self.session.lock().await;
        let s = session.as_mut().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = Uuid::parse_str(&params.uuid).map_err(McpServerError::Uuid)?;

        s.project
            .recover_from_trash(uuid)
            .map_err(McpServerError::Scrivener)?;

        let project = s.project.clone();
        tokio::task::spawn_blocking(move || project.save())
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        Ok(format!("Document recovered: {}", params.uuid))
    }

    async fn do_compile_documents(
        &self,
        params: CompileDocumentsParams,
    ) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let format = params.format.as_deref().unwrap_or("text");
        let compile_only = params.compile_only.unwrap_or(true);

        let items = if let Some(uuid_str) = &params.folder_uuid {
            let uuid = Uuid::parse_str(uuid_str).map_err(McpServerError::Uuid)?;
            match s.project.binder.find_by_uuid(uuid) {
                Some(scrivener::BinderItem::Folder(f)) => f.children.clone(),
                _ => {
                    return Err(McpServerError::DocumentNotFound {
                        identifier: uuid_str.clone(),
                    })
                }
            }
        } else {
            s.project
                .binder
                .root
                .iter()
                .find_map(|item| {
                    if let scrivener::BinderItem::Folder(f) = item {
                        Some(f.children.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_default()
        };

        let pp = s.project.path.clone();
        let fmt = format.to_string();
        let compiled =
            tokio::task::spawn_blocking(move || compile_items(&items, &pp, compile_only, &fmt))
                .await
                .map_err(|e| McpServerError::InvalidParameter {
                    message: e.to_string(),
                })?;

        Ok(compiled)
    }

    async fn do_export_project(&self, params: ExportProjectParams) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let format = params.format.as_deref().unwrap_or("text");
        let project = s.project.clone();
        let fmt = format.to_string();

        let compiled = tokio::task::spawn_blocking(move || {
            let mut output = String::new();
            if fmt == "markdown" {
                output.push_str(&format!("# {}\n\n", project.metadata.title));
                if let Some(author) = &project.metadata.author {
                    output.push_str(&format!("*By {}*\n\n---\n\n", author));
                }
            } else {
                output.push_str(&format!("{}\n", project.metadata.title));
                if let Some(author) = &project.metadata.author {
                    output.push_str(&format!("By {}\n", author));
                }
                output.push_str("\n---\n\n");
            }
            for item in &project.binder.root {
                if let scrivener::BinderItem::Folder(folder) = item {
                    output.push_str(&compile_items(&folder.children, &project.path, true, &fmt));
                }
            }
            output
        })
        .await
        .map_err(|e| McpServerError::InvalidParameter {
            message: e.to_string(),
        })?;

        Ok(compiled)
    }

    async fn do_get_statistics(&self) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let project = s.project.clone();

        let stats = tokio::task::spawn_blocking(move || project.statistics())
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?;

        let flat = s.project.binder.flatten();
        let words_by_document: Vec<DocumentWordCount> = flat
            .iter()
            .filter_map(|(item, _)| {
                if let scrivener::BinderItem::Document(doc) = item {
                    let wc = stats.words_by_document.get(&doc.uuid).copied().unwrap_or(0);
                    Some(DocumentWordCount {
                        uuid: doc.uuid.to_string(),
                        title: doc.title.clone(),
                        word_count: wc,
                    })
                } else {
                    None
                }
            })
            .collect();

        let ws = WritingStats {
            total_words: stats.total_words,
            total_characters: stats.total_characters,
            total_documents: stats.total_documents,
            total_folders: stats.total_folders,
            words_by_document,
        };

        Ok(serde_json::to_string_pretty(&ws)?)
    }

    async fn do_analyze_document(
        &self,
        params: AnalyzeDocumentParams,
    ) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = find_doc_uuid(&s.project.binder, &params.identifier)?;

        let doc = match s.project.binder.find_by_uuid(uuid) {
            Some(scrivener::BinderItem::Document(d)) => d.clone(),
            _ => {
                return Err(McpServerError::InvalidParameter {
                    message: "Cannot analyze a folder".into(),
                })
            }
        };

        let pp = s.project.path.clone();
        let content = tokio::task::spawn_blocking(move || doc.read_content(&pp))
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        let text = content.plain_text.unwrap_or_default();
        if text.trim().is_empty() {
            return Ok("Document is empty, no analysis available.".to_string());
        }

        let analyses = params.analyses;
        let result =
            tokio::task::spawn_blocking(move || build_analysis(&text, analyses.as_deref()))
                .await
                .map_err(|e| McpServerError::InvalidParameter {
                    message: e.to_string(),
                })?;

        Ok(result)
    }

    async fn do_get_word_count(&self, params: GetWordCountParams) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;

        if let Some(identifier) = &params.identifier {
            let uuid = find_doc_uuid(&s.project.binder, identifier)?;
            let doc = match s.project.binder.find_by_uuid(uuid) {
                Some(scrivener::BinderItem::Document(d)) => d.clone(),
                _ => {
                    return Err(McpServerError::InvalidParameter {
                        message: "Cannot count words for a folder".into(),
                    })
                }
            };

            let pp = s.project.path.clone();
            let content = tokio::task::spawn_blocking(move || doc.read_content(&pp))
                .await
                .map_err(|e| McpServerError::InvalidParameter {
                    message: e.to_string(),
                })?
                .map_err(McpServerError::Scrivener)?;

            let text = content.plain_text.unwrap_or_default();
            let result = serde_json::json!({
                "word_count": text.split_whitespace().count(),
                "character_count": text.chars().count(),
            });
            Ok(serde_json::to_string_pretty(&result)?)
        } else {
            let project = s.project.clone();
            let stats = tokio::task::spawn_blocking(move || project.statistics())
                .await
                .map_err(|e| McpServerError::InvalidParameter {
                    message: e.to_string(),
                })?;

            let result = serde_json::json!({
                "total_words": stats.total_words,
                "total_characters": stats.total_characters,
                "total_documents": stats.total_documents,
            });
            Ok(serde_json::to_string_pretty(&result)?)
        }
    }

    async fn do_analyze_readability(
        &self,
        params: AnalyzeReadabilityParams,
    ) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let uuid = find_doc_uuid(&s.project.binder, &params.identifier)?;

        let doc = match s.project.binder.find_by_uuid(uuid) {
            Some(scrivener::BinderItem::Document(d)) => d.clone(),
            _ => {
                return Err(McpServerError::InvalidParameter {
                    message: "Cannot analyze a folder".into(),
                })
            }
        };

        let pp = s.project.path.clone();
        let content = tokio::task::spawn_blocking(move || doc.read_content(&pp))
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?
            .map_err(McpServerError::Scrivener)?;

        let text = content.plain_text.unwrap_or_default();
        if text.trim().is_empty() {
            return Ok("Document is empty".to_string());
        }

        let result =
            tokio::task::spawn_blocking(move || {
                match writing_analysis::analyze_readability(&text) {
                    Ok(scores) => serde_json::to_string_pretty(&serde_json::json!({
                        "flesch_kincaid_grade": scores.flesch_kincaid_grade,
                        "flesch_reading_ease": scores.flesch_reading_ease,
                        "smog_index": scores.smog_index,
                        "coleman_liau_index": scores.coleman_liau_index,
                        "automated_readability_index": scores.automated_readability_index,
                    }))
                    .unwrap_or_default(),
                    Err(e) => format!("Analysis error: {}", e),
                }
            })
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?;

        Ok(result)
    }

    async fn do_update_memory(&self, params: UpdateMemoryParams) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let pp = s.project_path.display().to_string();
        let category = params.category.as_deref().unwrap_or("general");

        self.database
            .upsert_memory(&pp, &params.key, &params.value, category)?;
        Ok(format!(
            "Memory updated: key='{}', category='{}'",
            params.key, category
        ))
    }

    async fn do_get_memory(&self, params: GetMemoryParams) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let pp = s.project_path.display().to_string();

        let entries =
            self.database
                .get_memory(&pp, params.key.as_deref(), params.category.as_deref())?;

        if entries.len() == 1 {
            Ok(serde_json::to_string_pretty(&entries[0])?)
        } else {
            Ok(serde_json::to_string_pretty(&entries)?)
        }
    }

    async fn do_check_consistency(
        &self,
        _params: CheckConsistencyParams,
    ) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let pp = s.project_path.display().to_string();

        let entries = self.database.get_memory(&pp, None, None)?;
        if entries.is_empty() {
            return Ok(
                "No memory entries found. Use update_memory to store project information first."
                    .to_string(),
            );
        }

        let report = serde_json::json!({
            "memory_entries": entries.len(),
            "categories": entries.iter().map(|e| e.category.as_str()).collect::<std::collections::HashSet<_>>(),
            "note": "Store character profiles, plot points, and details with update_memory to enable consistency checks.",
        });
        Ok(serde_json::to_string_pretty(&report)?)
    }

    async fn do_get_project_summary(&self) -> crate::error::Result<String> {
        let session = self.session.lock().await;
        let s = session.as_ref().ok_or(McpServerError::NoProjectOpen)?;
        let pp = s.project_path.display().to_string();

        let project = s.project.clone();
        let stats = tokio::task::spawn_blocking(move || project.statistics())
            .await
            .map_err(|e| McpServerError::InvalidParameter {
                message: e.to_string(),
            })?;

        let entries = self.database.get_memory(&pp, None, None)?;

        let summary = ProjectSummary {
            project_info: ProjectInfo {
                title: s.project.metadata.title.clone(),
                author: s.project.metadata.author.clone(),
                path: pp,
                document_count: stats.total_documents,
                folder_count: stats.total_folders,
                total_words: stats.total_words,
            },
            memory_entries: entries,
        };

        Ok(serde_json::to_string_pretty(&summary)?)
    }
}
