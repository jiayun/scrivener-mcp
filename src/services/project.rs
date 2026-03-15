use std::path::PathBuf;

/// Represents an open Scrivener project session.
pub struct ProjectSession {
    /// The loaded Scrivener project.
    pub project: scrivener::Project,

    /// Absolute path to the .scriv bundle.
    pub project_path: PathBuf,

    /// When this session was started.
    pub opened_at: chrono::DateTime<chrono::Utc>,
}
