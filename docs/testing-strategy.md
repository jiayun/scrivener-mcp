# Testing Strategy

## Test Organization

```
tests/
├── integration/
│   ├── mod.rs                    # Common test helpers, fixtures
│   ├── project_tools_test.rs     # Project lifecycle tool tests
│   ├── document_tools_test.rs    # Document CRUD tool tests
│   ├── search_tools_test.rs      # Search and trash tool tests
│   ├── compile_tools_test.rs     # Compilation tool tests
│   ├── analysis_tools_test.rs    # Writing analysis tool tests
│   └── memory_tools_test.rs      # Memory persistence tool tests
└── fixtures/
    └── sample.scriv/             # Sample Scrivener project (from scrivener-rs)
        ├── sample.scrivx
        ├── Files/
        │   └── Data/
        │       ├── 11111111-1111-1111-1111-111111111111/
        │       │   ├── content.rtf
        │       │   ├── notes.rtf
        │       │   └── synopsis.txt
        │       ├── 22222222-2222-2222-2222-222222222222/
        │       │   └── content.rtf
        │       └── 33333333-3333-3333-3333-333333333333/
        │           └── content.rtf
        └── Settings/

# Unit tests are in-module via #[cfg(test)] in each src/*.rs file
```

## Test Approach

### Integration Tests: MCP Tool Round-trip

Integration tests call tool handler functions directly with constructed parameters, verifying the full pipeline from parameter deserialization to response content.

```rust
use scrivener_mcp::server::ScrivenerMcp;
use rmcp::model::*;
use serde_json::json;
use tempfile::TempDir;

/// Helper: create a server instance with a temp database
async fn setup_server() -> (ScrivenerMcp, TempDir) {
    let temp = TempDir::new().unwrap();
    let db_path = temp.path().join("test.db");
    let server = ScrivenerMcp::new(db_path).await.unwrap();
    (server, temp)
}

/// Helper: copy fixture to temp dir for mutation tests
fn copy_fixture_to_temp(temp: &TempDir) -> PathBuf {
    let src = Path::new("tests/fixtures/sample.scriv");
    let dest = temp.path().join("sample.scriv");
    copy_dir_recursive(src, &dest).unwrap();
    dest
}

/// Helper: call a tool and extract the text content from the result
async fn call_tool(server: &ScrivenerMcp, name: &str, args: serde_json::Value) -> String {
    let request = CallToolRequestParam {
        name: name.into(),
        arguments: args.as_object().cloned(),
    };
    let result = server.call_tool(request, /* context */).await.unwrap();
    // Extract text from first content block
    match &result.content[0] {
        Content::Text(t) => t.text.clone(),
        _ => panic!("Expected text content"),
    }
}
```

### Unit Tests: Per-Module

Each handler and service module has in-module `#[cfg(test)]` unit tests for isolated logic.

## Test Cases

### Project Tools (`project_tools_test.rs`)

```rust
#[tokio::test]
async fn open_project_success() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);

    let result = call_tool(&server, "open_project", json!({
        "path": fixture_path.to_str().unwrap()
    })).await;

    let info: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(info["title"], "Sample Novel");
    assert_eq!(info["author"], "Test Author");
}

#[tokio::test]
async fn open_project_not_found_error() {
    let (server, _temp) = setup_server().await;

    let result = server.call_tool(CallToolRequestParam {
        name: "open_project".into(),
        arguments: json!({"path": "/nonexistent/path.scriv"}).as_object().cloned(),
    }, /* context */).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn close_project_clears_session() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);

    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;
    call_tool(&server, "close_project", json!({})).await;

    // Subsequent tool calls should fail with NoProjectOpen
    let result = server.call_tool(CallToolRequestParam {
        name: "get_structure".into(),
        arguments: None,
    }, /* context */).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn get_structure_returns_binder_tree() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);

    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;
    let result = call_tool(&server, "get_structure", json!({})).await;

    let structure: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(structure.is_array());
    // Should contain Draft and Research folders
    let titles: Vec<&str> = structure.as_array().unwrap()
        .iter()
        .map(|item| item["title"].as_str().unwrap())
        .collect();
    assert!(titles.contains(&"Draft"));
    assert!(titles.contains(&"Research"));
}

#[tokio::test]
async fn refresh_project_reloads_from_disk() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);

    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;
    call_tool(&server, "refresh_project", json!({})).await;

    // Should still be able to get structure after refresh
    let result = call_tool(&server, "get_structure", json!({})).await;
    let structure: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(structure.is_array());
}
```

### Document Tools (`document_tools_test.rs`)

```rust
#[tokio::test]
async fn read_document_by_uuid() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "read_document", json!({
        "identifier": "11111111-1111-1111-1111-111111111111"
    })).await;

    assert!(result.contains("dark and stormy night"));
}

#[tokio::test]
async fn read_document_by_title() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "read_document", json!({
        "identifier": "Chapter One"
    })).await;

    assert!(result.contains("dark and stormy night"));
}

#[tokio::test]
async fn write_and_read_roundtrip() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    call_tool(&server, "write_document", json!({
        "identifier": "11111111-1111-1111-1111-111111111111",
        "content": "New content for testing."
    })).await;

    let result = call_tool(&server, "read_document", json!({
        "identifier": "11111111-1111-1111-1111-111111111111"
    })).await;

    assert!(result.contains("New content for testing"));
}

#[tokio::test]
async fn create_document_in_draft() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "create_document", json!({
        "title": "New Chapter",
        "content": "Once upon a time..."
    })).await;

    let info: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(info["title"], "New Chapter");
    assert!(info["uuid"].as_str().is_some());
}

#[tokio::test]
async fn delete_document_moves_to_trash() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    call_tool(&server, "delete_document", json!({
        "identifier": "22222222-2222-2222-2222-222222222222"
    })).await;

    let trash = call_tool(&server, "list_trash", json!({})).await;
    assert!(trash.contains("Chapter Two"));
}

#[tokio::test]
async fn update_metadata_synopsis_and_keywords() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    call_tool(&server, "update_metadata", json!({
        "identifier": "11111111-1111-1111-1111-111111111111",
        "synopsis": "A dark and stormy beginning",
        "add_keywords": ["weather", "night"]
    })).await;

    let info = call_tool(&server, "get_document_info", json!({
        "identifier": "11111111-1111-1111-1111-111111111111"
    })).await;

    let doc: serde_json::Value = serde_json::from_str(&info).unwrap();
    assert_eq!(doc["synopsis"], "A dark and stormy beginning");
    assert!(doc["keywords"].as_array().unwrap().contains(&json!("weather")));
}
```

### Search Tools (`search_tools_test.rs`)

```rust
#[tokio::test]
async fn search_content_finds_matches() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "search_content", json!({
        "query": "protagonist"
    })).await;

    let results: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(!results.as_array().unwrap().is_empty());
    assert_eq!(results[0]["document_title"], "Chapter One");
}

#[tokio::test]
async fn search_no_results_returns_empty() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "search_content", json!({
        "query": "xyzzy_nonexistent_term"
    })).await;

    let results: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(results.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn list_trash_shows_deleted_items() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "list_trash", json!({})).await;
    let items: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(items.as_array().unwrap().len(), 1);
    assert_eq!(items[0]["title"], "Deleted Scene");
}

#[tokio::test]
async fn recover_document_from_trash() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    call_tool(&server, "recover_document", json!({
        "uuid": "44444444-4444-4444-4444-444444444444"
    })).await;

    let trash = call_tool(&server, "list_trash", json!({})).await;
    let items: serde_json::Value = serde_json::from_str(&trash).unwrap();
    assert!(items.as_array().unwrap().is_empty());
}
```

### Analysis Tools (`analysis_tools_test.rs`)

```rust
#[tokio::test]
async fn analyze_document_returns_all_metrics() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "analyze_document", json!({
        "identifier": "11111111-1111-1111-1111-111111111111"
    })).await;

    let analysis: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(analysis.get("readability").is_some());
    assert!(analysis.get("passive_voice").is_some());
    assert!(analysis.get("sentiment").is_some());
}

#[tokio::test]
async fn analyze_document_selective_analyses() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "analyze_document", json!({
        "identifier": "11111111-1111-1111-1111-111111111111",
        "analyses": ["readability", "sentiment"]
    })).await;

    let analysis: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(analysis.get("readability").is_some());
    assert!(analysis.get("sentiment").is_some());
}

#[tokio::test]
async fn get_word_count_single_document() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "get_word_count", json!({
        "identifier": "11111111-1111-1111-1111-111111111111"
    })).await;

    let count: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(count["word_count"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn analyze_readability_returns_scores() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    let result = call_tool(&server, "analyze_readability", json!({
        "identifier": "11111111-1111-1111-1111-111111111111"
    })).await;

    let scores: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(scores.get("flesch_kincaid_grade").is_some());
    assert!(scores.get("flesch_reading_ease").is_some());
}
```

### Memory Tools (`memory_tools_test.rs`)

```rust
#[tokio::test]
async fn update_and_get_memory() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    call_tool(&server, "update_memory", json!({
        "key": "protagonist_name",
        "value": "John Smith",
        "category": "character"
    })).await;

    let result = call_tool(&server, "get_memory", json!({
        "key": "protagonist_name"
    })).await;

    let memory: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(memory["value"], "John Smith");
    assert_eq!(memory["category"], "character");
}

#[tokio::test]
async fn get_memory_by_category() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    call_tool(&server, "update_memory", json!({"key": "hero", "value": "John", "category": "character"})).await;
    call_tool(&server, "update_memory", json!({"key": "villain", "value": "Jane", "category": "character"})).await;
    call_tool(&server, "update_memory", json!({"key": "setting", "value": "London", "category": "setting"})).await;

    let result = call_tool(&server, "get_memory", json!({
        "category": "character"
    })).await;

    let memories: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(memories.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn memory_persists_across_sessions() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    call_tool(&server, "update_memory", json!({"key": "test_key", "value": "test_value"})).await;
    call_tool(&server, "close_project", json!({})).await;

    // Re-open the same project
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;
    let result = call_tool(&server, "get_memory", json!({"key": "test_key"})).await;

    let memory: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(memory["value"], "test_value");
}

#[tokio::test]
async fn update_memory_overwrites_existing() {
    let (server, temp) = setup_server().await;
    let fixture_path = copy_fixture_to_temp(&temp);
    call_tool(&server, "open_project", json!({"path": fixture_path.to_str().unwrap()})).await;

    call_tool(&server, "update_memory", json!({"key": "name", "value": "v1"})).await;
    call_tool(&server, "update_memory", json!({"key": "name", "value": "v2"})).await;

    let result = call_tool(&server, "get_memory", json!({"key": "name"})).await;
    let memory: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(memory["value"], "v2");
}
```

## Test Coverage Goals

| Module | Coverage Target | Key Scenarios |
|--------|----------------|---------------|
| `handlers/project.rs` | 90%+ | open, close, refresh, structure, errors (not found, already open) |
| `handlers/document.rs` | 90%+ | read (UUID/title), write, create, delete, rename, move, metadata |
| `handlers/search.rs` | 85%+ | text search, regex, trash list, trash search, recover |
| `handlers/compile.rs` | 85%+ | compile draft, export, statistics |
| `handlers/analysis.rs` | 85%+ | full analysis, selective, readability, word count |
| `handlers/memory.rs` | 90%+ | CRUD, categories, persistence, overwrite |
| `services/database.rs` | 85%+ | table creation, insert, query, update, delete |
| `services/project.rs` | 80%+ | session lifecycle, state transitions |
| `server.rs` | 80%+ | initialization, tool routing, error handling |
| `error.rs` | 90%+ | all error variants, MCP error mapping |

## Temp Directory Pattern

All tests use `tempfile::TempDir` to ensure isolation and prevent fixture mutation:

```rust
fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dest)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let target = dest.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_recursive(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}
```

## CI Considerations

- Tests run with `cargo test` — no external services required
- SQLite uses in-memory or temp-dir databases
- Fixture `.scriv` project is committed to the repository
- `cargo clippy` runs in CI with `-- -D warnings`
- Tests should complete in < 30 seconds
