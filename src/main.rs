mod error;
mod server;
mod services;
mod types;

use std::path::PathBuf;

use clap::Parser;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(
    name = "scrivener-mcp",
    version,
    about = "MCP server for Scrivener 3 projects"
)]
struct Cli {
    /// Path to the SQLite database file
    #[arg(long, default_value = "~/.scrivener-mcp/data.db")]
    db_path: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Init tracing to stderr (stdout is reserved for JSON-RPC)
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&cli.log_level)),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("scrivener-mcp v{} starting", env!("CARGO_PKG_VERSION"));

    // Resolve db path
    let db_path = if cli.db_path.starts_with('~') {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(cli.db_path.replacen('~', &home, 1))
    } else {
        PathBuf::from(&cli.db_path)
    };

    let database = services::database::Database::open(&db_path)?;
    tracing::info!("Database opened at {}", db_path.display());

    let server = server::ScrivenerMcp::new(database);

    let transport = rmcp::transport::io::stdio();

    tracing::info!("Starting MCP server on stdio");
    let service = server.serve(transport).await?;
    service.waiting().await?;

    tracing::info!("Server shutdown");
    Ok(())
}
