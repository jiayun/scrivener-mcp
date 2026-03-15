use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpServerError {
    #[error("No project is currently open")]
    NoProjectOpen,

    #[error("Project already open: {path}")]
    ProjectAlreadyOpen { path: String },

    #[error("Document not found: {identifier}")]
    DocumentNotFound { identifier: String },

    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

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

    #[error("UUID parse error: {0}")]
    Uuid(#[from] uuid::Error),
}

impl From<McpServerError> for rmcp::ErrorData {
    fn from(e: McpServerError) -> Self {
        rmcp::ErrorData {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: e.to_string().into(),
            data: None,
        }
    }
}

pub type Result<T> = std::result::Result<T, McpServerError>;
