# scrivener-mcp

MCP server for [Scrivener 3](https://www.literatureandlatte.com/scrivener/overview) projects — AI-powered writing assistant tools.

Provides 28 tools that let AI assistants (Claude, etc.) read, write, analyze, and manage Scrivener projects through the [Model Context Protocol](https://modelcontextprotocol.io/).

## Features

- **Project management** — open/close/refresh projects, view binder structure
- **Document operations** — read, write, create, delete, rename, move documents
- **Search** — full-text search across all documents
- **Trash management** — list, search, and recover deleted documents
- **Compilation** — compile documents in reading order, export entire draft
- **Writing analysis** — readability scores (Flesch-Kincaid, SMOG, Coleman-Liau, ARI), passive voice, clichés, filter words, sentiment, sentence variety
- **Statistics** — project-wide and per-document word counts
- **Project memory** — persistent notes per project with consistency checking

## Installation

### From crates.io

```sh
cargo install scrivener-mcp
```

### From GitHub Releases

Download a prebuilt binary from [Releases](https://github.com/jiayun/scrivener-mcp/releases) for your platform:

| Platform | File |
|----------|------|
| macOS (Apple Silicon) | `scrivener-mcp-aarch64-apple-darwin.tar.gz` |
| macOS (Intel) | `scrivener-mcp-x86_64-apple-darwin.tar.gz` |
| Linux (x86_64) | `scrivener-mcp-x86_64-unknown-linux-gnu.tar.gz` |
| Windows (x86_64) | `scrivener-mcp-x86_64-pc-windows-msvc.zip` |

> **macOS:** The downloaded binary is unsigned. Remove the quarantine attribute before running:
>
> ```sh
> xattr -d com.apple.quarantine scrivener-mcp
> ```
>
> **Windows:** SmartScreen may show "Windows protected your PC". Click **More info → Run anyway** to proceed.

### Build from source

```sh
git clone https://github.com/jiayun/scrivener-mcp.git
cd scrivener-mcp
cargo install --path .
```

## Configuration

### Claude Desktop

Add to your Claude Desktop config (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "scrivener": {
      "command": "scrivener-mcp",
      "args": []
    }
  }
}
```

### Claude Code

Add to your project's `.mcp.json`:

```json
{
  "mcpServers": {
    "scrivener": {
      "command": "scrivener-mcp",
      "args": []
    }
  }
}
```

Or add interactively:

```sh
claude mcp add scrivener scrivener-mcp
```

## Usage

1. **Open a project** — point the server at your `.scriv` bundle
2. **Browse structure** — view the binder hierarchy
3. **Read & write** — access document content by title or UUID
4. **Analyze writing** — get readability scores, find passive voice, clichés, etc.
5. **Compile & export** — combine documents into a single output

## CLI Options

```
scrivener-mcp [OPTIONS]

Options:
  --db_path <PATH>    Path to the SQLite database file [default: ~/.scrivener-mcp/data.db]
  --log_level <LEVEL> Log level: trace, debug, info, warn, error [default: info]
  -h, --help          Print help
  -V, --version       Print version
```

## Tools

| Category | Tool | Description |
|----------|------|-------------|
| Project | `open_project` | Open a Scrivener 3 project (.scriv bundle) |
| Project | `close_project` | Close the currently open project |
| Project | `refresh_project` | Reload project from disk |
| Project | `get_structure` | Get hierarchical binder structure |
| Project | `get_project_summary` | Comprehensive project summary |
| Project | `get_session_info` | Current MCP server session info |
| Document | `read_document` | Read document content by UUID or title |
| Document | `write_document` | Write content to a document |
| Document | `create_document` | Create a new document |
| Document | `delete_document` | Move document to trash |
| Document | `rename_document` | Rename a document or folder |
| Document | `move_document` | Move document to a different folder |
| Document | `get_document_info` | Detailed document info and metadata |
| Document | `update_metadata` | Update synopsis, notes, keywords |
| Search | `search_content` | Full-text search across documents |
| Trash | `list_trash` | List documents in trash |
| Trash | `search_trash` | Search trash by title |
| Trash | `recover_document` | Recover document from trash |
| Compile | `compile_documents` | Compile documents in reading order |
| Compile | `export_project` | Export entire draft as single document |
| Analysis | `analyze_document` | Full writing analysis (readability, style, sentiment) |
| Analysis | `analyze_readability` | Readability scores |
| Analysis | `check_consistency` | Check memory against document state |
| Stats | `get_statistics` | Project statistics and breakdown |
| Stats | `get_word_count` | Word count for document or project |
| Stats | `get_writing_stats` | Detailed per-document writing stats |
| Memory | `update_memory` | Store/update persistent project memory |
| Memory | `get_memory` | Retrieve stored memory entries |

## License

MIT
