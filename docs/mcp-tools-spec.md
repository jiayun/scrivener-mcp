# MCP Tools Specification

This document defines all 29 MCP tools for Phase 2 of scrivener-mcp. Each tool maps to underlying `scrivener` or `writing-analysis` crate APIs.

---

## Project Tools (4)

### open_project

Opens a Scrivener 3 project from disk and creates a session.

```rust
#[tool(name = "open_project", description = "Open a Scrivener 3 project (.scriv bundle) from the specified path")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Absolute path to the .scriv bundle directory"
    }
  },
  "required": ["path"]
}
```

**Output:** JSON with project info (title, author, document count, word count).

**Maps to:** `scrivener::Project::open(path)`

---

### close_project

Closes the currently open project and clears the session.

```rust
#[tool(name = "close_project", description = "Close the currently open Scrivener project")]
```

**Input Schema:** No parameters.

**Output:** Confirmation message.

**Maps to:** Drop `ProjectSession`, log to session history.

---

### refresh_project

Reloads the current project from disk, picking up any external changes.

```rust
#[tool(name = "refresh_project", description = "Reload the current project from disk to pick up external changes")]
```

**Input Schema:** No parameters.

**Output:** Updated project info.

**Maps to:** `Project::open()` on the same path, replacing the session.

---

### get_structure

Returns the hierarchical binder structure of the project.

```rust
#[tool(name = "get_structure", description = "Get the hierarchical binder structure showing all documents and folders")]
```

**Input Schema:** No parameters.

**Output:** JSON array of `BinderItemInfo` (recursive tree with uuid, title, type, children, include_in_compile).

**Maps to:** `Project.binder.root` traversal → `BinderItemInfo` conversion.

---

## Document Tools (9)

### read_document

Reads the plain text content of a document.

```rust
#[tool(name = "read_document", description = "Read the text content of a document by UUID or title")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title"
    },
    "max_length": {
      "type": "integer",
      "description": "Maximum content length to return (optional)"
    }
  },
  "required": ["identifier"]
}
```

**Output:** Document plain text content.

**Maps to:** `Binder::find_by_uuid()` or `find_by_title()` → `Document::read_content()` → `plain_text`

---

### write_document

Writes new content to an existing document.

```rust
#[tool(name = "write_document", description = "Write new text content to a document (replaces existing content)")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title"
    },
    "content": {
      "type": "string",
      "description": "New content to write (plain text, converted to RTF)"
    }
  },
  "required": ["identifier", "content"]
}
```

**Output:** Confirmation with word count of new content.

**Maps to:** `Document::write_content(content)` → `Project::save()`

---

### create_document

Creates a new document in the project.

```rust
#[tool(name = "create_document", description = "Create a new document in the specified folder (defaults to Draft)")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "title": {
      "type": "string",
      "description": "Title for the new document"
    },
    "parent_uuid": {
      "type": "string",
      "description": "UUID of parent folder (optional, defaults to Draft)"
    },
    "content": {
      "type": "string",
      "description": "Initial content (optional)"
    }
  },
  "required": ["title"]
}
```

**Output:** JSON with new document's UUID, title, and path.

**Maps to:** Create `Document` → add to binder → write content → `Project::save()`

---

### create_folder

Creates a new folder in the project.

```rust
#[tool(name = "create_folder", description = "Create a new folder in the specified parent folder (defaults to Draft)")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "title": {
      "type": "string",
      "description": "Title for the new folder"
    },
    "parent_uuid": {
      "type": "string",
      "description": "UUID of parent folder (optional, defaults to Draft)"
    }
  },
  "required": ["title"]
}
```

**Output:** JSON with new folder's UUID and title.

**Maps to:** Create `Folder` → add to binder → `Project::save()`

---

### delete_document

Moves a document to the trash.

```rust
#[tool(name = "delete_document", description = "Move a document to the trash (can be recovered later)")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title to delete"
    }
  },
  "required": ["identifier"]
}
```

**Output:** Confirmation message.

**Maps to:** Remove from binder → add to trash → `Project::save()`

---

### rename_document

Renames a document or folder.

```rust
#[tool(name = "rename_document", description = "Rename a document or folder")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title"
    },
    "new_title": {
      "type": "string",
      "description": "New title"
    }
  },
  "required": ["identifier", "new_title"]
}
```

**Output:** Confirmation with old and new title.

**Maps to:** Update `Document.title` or `Folder.title` → `Project::save()`

---

### move_document

Moves a document or folder to a different parent in the binder.

```rust
#[tool(name = "move_document", description = "Move a document or folder to a different parent folder")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "uuid": {
      "type": "string",
      "description": "UUID of the item to move"
    },
    "target_parent_uuid": {
      "type": "string",
      "description": "UUID of target parent folder (omit to move to root)"
    }
  },
  "required": ["uuid"]
}
```

**Output:** Confirmation with new location.

**Maps to:** `Binder::move_item(uuid, target_parent)` → `Project::save()`

---

### get_document_info

Returns detailed metadata about a document.

```rust
#[tool(name = "get_document_info", description = "Get detailed information about a document including metadata, word count, and binder path")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title"
    }
  },
  "required": ["identifier"]
}
```

**Output:** JSON `DocumentInfo` (uuid, title, synopsis, keywords, word_count, dates, path).

**Maps to:** Binder lookup → `Document` fields + `DocumentMetadata` + `Binder::flatten()` for path.

---

### update_metadata

Updates document metadata (synopsis, notes, keywords).

```rust
#[tool(name = "update_metadata", description = "Update document metadata: synopsis, notes, and/or keywords")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title"
    },
    "synopsis": {
      "type": "string",
      "description": "New synopsis text"
    },
    "notes": {
      "type": "string",
      "description": "New notes text"
    },
    "add_keywords": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Keywords to add"
    },
    "remove_keywords": {
      "type": "array",
      "items": { "type": "string" },
      "description": "Keywords to remove"
    }
  },
  "required": ["identifier"]
}
```

**Output:** Updated document info.

**Maps to:** `Document::update_synopsis()`, `update_notes()`, `add_keyword()`, `remove_keyword()` → `Project::save()`

---

## Search Tools (4)

### search_content

Searches for text across all documents in the project.

```rust
#[tool(name = "search_content", description = "Search for text content across all documents in the project")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Search query string"
    },
    "regex": {
      "type": "boolean",
      "description": "Use regex matching (default: false)"
    },
    "max_results": {
      "type": "integer",
      "description": "Maximum number of results (default: 50)"
    }
  },
  "required": ["query"]
}
```

**Output:** JSON array of `SearchResultItem` with context snippets.

**Maps to:** `Project::search(query)` or `Project::search_regex(pattern)`

---

### list_trash

Lists all documents currently in the trash.

```rust
#[tool(name = "list_trash", description = "List all documents currently in the trash")]
```

**Input Schema:** No parameters.

**Output:** JSON array of trashed items (uuid, title, type).

**Maps to:** `Project::list_trash()`

---

### search_trash

Searches for documents in the trash by title or content.

```rust
#[tool(name = "search_trash", description = "Search for documents in the trash by title")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "query": {
      "type": "string",
      "description": "Search query string"
    }
  },
  "required": ["query"]
}
```

**Output:** JSON array of matching trashed items.

**Maps to:** Filter `Trash.items` by title match.

---

### recover_document

Recovers a document from the trash back into the binder.

```rust
#[tool(name = "recover_document", description = "Recover a document from the trash back into the project binder")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "uuid": {
      "type": "string",
      "description": "UUID of the trashed document to recover"
    }
  },
  "required": ["uuid"]
}
```

**Output:** Confirmation with recovered document info.

**Maps to:** `Project::recover_from_trash(uuid)` → `Project::save()`

---

## Compilation Tools (3)

### compile_documents

Compiles documents in reading order into a single output.

```rust
#[tool(name = "compile_documents", description = "Compile documents in reading order into a single text output")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "folder_uuid": {
      "type": "string",
      "description": "UUID of folder to compile (default: Draft)"
    },
    "format": {
      "type": "string",
      "enum": ["text", "markdown"],
      "description": "Output format (default: text)"
    },
    "compile_only": {
      "type": "boolean",
      "description": "Only include documents marked for compile (default: true)"
    }
  }
}
```

**Output:** Compiled text content.

**Maps to:** Traverse binder folder → collect documents → `read_content()` → concatenate.

---

### export_project

Exports the entire project as a single formatted output.

```rust
#[tool(name = "export_project", description = "Export the entire project draft as a single document")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "format": {
      "type": "string",
      "enum": ["text", "markdown"],
      "description": "Output format (default: text)"
    }
  }
}
```

**Output:** Full project text.

**Maps to:** Same as `compile_documents` on Draft folder with headers.

---

### get_statistics

Returns project-wide statistics.

```rust
#[tool(name = "get_statistics", description = "Get project statistics: document count, word count, and per-document breakdown")]
```

**Input Schema:** No parameters.

**Output:** JSON `WritingStats` (total words, characters, documents, per-document breakdown).

**Maps to:** `Project::statistics()`

---

## Analysis Tools (3)

### analyze_document

Runs writing analysis on a document's content.

```rust
#[tool(name = "analyze_document", description = "Analyze a document for readability, passive voice, clichés, filter words, sentiment, and sentence variety")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title"
    },
    "analyses": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": ["readability", "passive_voice", "cliches", "filter_words", "sentiment", "sentence_variety"]
      },
      "description": "Specific analyses to run (default: all)"
    }
  },
  "required": ["identifier"]
}
```

**Output:** JSON with analysis results for each requested type.

**Maps to:** `Document::read_content()` → `writing_analysis::analyze_all()` or individual analyzers.

---

### get_word_count

Returns word count for a document or the entire project.

```rust
#[tool(name = "get_word_count", description = "Get word count for a specific document or the entire project")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title (omit for project-wide count)"
    }
  }
}
```

**Output:** JSON with word_count, character_count, sentence_count.

**Maps to:** `Document::read_content()` → count, or `Project::statistics()` for project-wide.

---

### analyze_readability

Returns readability scores for a document.

```rust
#[tool(name = "analyze_readability", description = "Analyze readability of a document (Flesch-Kincaid, SMOG, Coleman-Liau, ARI)")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "identifier": {
      "type": "string",
      "description": "Document UUID or title"
    }
  },
  "required": ["identifier"]
}
```

**Output:** JSON `ReadabilityScores` (flesch_kincaid_grade, flesch_reading_ease, smog_index, coleman_liau_index, automated_readability_index).

**Maps to:** `Document::read_content()` → `writing_analysis::analyze_readability(text)`

---

## Memory Tools (4)

### update_memory

Stores or updates a key-value memory entry for the current project.

```rust
#[tool(name = "update_memory", description = "Store or update a memory entry for the current project (persists across sessions)")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "key": {
      "type": "string",
      "description": "Memory key (e.g., 'protagonist_profile', 'plot_outline')"
    },
    "value": {
      "type": "string",
      "description": "Memory value (free-form text)"
    },
    "category": {
      "type": "string",
      "description": "Category: character, plot, setting, theme, notes (default: general)"
    }
  },
  "required": ["key", "value"]
}
```

**Output:** Confirmation with timestamp.

**Maps to:** `Database::upsert_memory(project_path, key, value, category)`

---

### get_memory

Retrieves memory entries for the current project.

```rust
#[tool(name = "get_memory", description = "Retrieve stored memory entries for the current project")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "key": {
      "type": "string",
      "description": "Specific key to retrieve (omit for all)"
    },
    "category": {
      "type": "string",
      "description": "Filter by category"
    }
  }
}
```

**Output:** JSON `MemoryEntry` or array of entries.

**Maps to:** `Database::get_memory(project_path, key?, category?)`

---

### check_consistency

Checks stored memory against current project state for inconsistencies.

```rust
#[tool(name = "check_consistency", description = "Check stored project memory against current document state for inconsistencies")]
```

**Input Schema:**
```json
{
  "type": "object",
  "properties": {
    "aspects": {
      "type": "array",
      "items": {
        "type": "string",
        "enum": ["characters", "timeline", "locations", "plot"]
      },
      "description": "Aspects to check (default: all)"
    }
  }
}
```

**Output:** JSON report of consistency findings.

**Maps to:** Compare `Database::get_memory()` entries against current document content via search.

---

### get_project_summary

Returns a comprehensive project summary combining info, memory, and stats.

```rust
#[tool(name = "get_project_summary", description = "Get a comprehensive project summary including info, statistics, and stored memories")]
```

**Input Schema:** No parameters.

**Output:** JSON `ProjectSummary` (project_info, memory_entries, recent_sessions).

**Maps to:** Combine `Project` metadata + `statistics()` + `Database` queries.

---

## Stats Tools (2)

### get_writing_stats

Returns detailed writing statistics with per-document breakdown.

```rust
#[tool(name = "get_writing_stats", description = "Get detailed writing statistics with per-document word count breakdown")]
```

**Input Schema:** No parameters.

**Output:** JSON `WritingStats`.

**Maps to:** `Project::statistics()` → per-document `read_content()` word counts.

---

### get_session_info

Returns information about the current server session.

```rust
#[tool(name = "get_session_info", description = "Get information about the current MCP server session")]
```

**Input Schema:** No parameters.

**Output:** JSON with server version, uptime, current project path, session history.

**Maps to:** Server internal state + `Database::get_session_history()`.
