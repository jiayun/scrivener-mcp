# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/),
and this project adheres to [Semantic Versioning](https://semver.org/).

## [0.2.0] - 2026-04-24

### Added

- New `create_folder` tool for creating binder folders — previously only `create_document` was available, leaving no way to create a new chapter/part container. The new tool mirrors `create_document` semantics (defaults to Draft folder, accepts optional `parent_uuid`). Total tools: 29.

## [0.1.1] - 2026-03-19

### Added

- Custom metadata support in `update_metadata` tool — set or update user-defined metadata fields via `custom_metadata` parameter (fixes #1)

## [0.1.0] - 2026-03-15

### Added

- MCP server with 28 tools for Scrivener 3 project interaction
- **Project management**: open, close, refresh projects; get structure and session info
- **Document operations**: read, write, create, delete, rename, move documents; get document info and metadata updates
- **Search**: full-text content search across all documents
- **Trash management**: list, search, and recover deleted documents
- **Compilation**: compile documents in reading order; export entire project draft
- **Writing analysis**: readability scores (Flesch-Kincaid, SMOG, Coleman-Liau, ARI), passive voice detection, cliché finder, filter word analysis, sentiment analysis, sentence variety metrics
- **Statistics**: project-wide and per-document word counts, writing stats breakdown
- **Project memory**: persistent key-value memory per project with consistency checking against document state
- Cross-platform support: macOS (Intel & Apple Silicon), Linux, Windows
- SQLite-based persistent storage (bundled, no system dependencies)
- stdio transport for MCP protocol communication
