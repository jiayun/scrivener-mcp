# Tool Mapping: dcondrey/scrivener-mcp → scrivener-mcp

This document maps all tools from the [dcondrey/scrivener-mcp](https://github.com/dcondrey/scrivener-mcp) TypeScript reference implementation to our Rust scrivener-mcp Phase 2/3 plan.

**Legend:**
- ✅ Phase 2 — Core functionality, will be implemented
- ⏳ Phase 3 — Advanced features, deferred
- ❌ Not planned — Out of scope or covered by other tools

---

## Project Management

| # | dcondrey Tool | scrivener-mcp | Status | Notes |
|---|--------------|---------------|--------|-------|
| 1 | `open_project` | `open_project` | ✅ Phase 2 | Same functionality |
| 2 | `get_structure` | `get_structure` | ✅ Phase 2 | Same functionality |
| 3 | `refresh_project` | `refresh_project` | ✅ Phase 2 | Same functionality |
| 4 | `close_project` | `close_project` | ✅ Phase 2 | Same functionality |

## Document Operations

| # | dcondrey Tool | scrivener-mcp | Status | Notes |
|---|--------------|---------------|--------|-------|
| 5 | `get_document_info` | `get_document_info` | ✅ Phase 2 | Same functionality |
| 6 | `read_document` | `read_document` | ✅ Phase 2 | Same functionality |
| 7 | `write_document` | `write_document` | ✅ Phase 2 | Same functionality |
| 8 | `create_document` | `create_document` | ✅ Phase 2 | Same functionality |
| 9 | `delete_document` | `delete_document` | ✅ Phase 2 | Same functionality |
| 10 | `rename_document` | `rename_document` | ✅ Phase 2 | Same functionality |
| 11 | `move_document` | `move_document` | ✅ Phase 2 | Same functionality |
| 12 | `update_metadata` | `update_metadata` | ✅ Phase 2 | Same functionality |
| 13 | `get_word_count` | `get_word_count` | ✅ Phase 2 | Same functionality |
| 14 | `read_document_formatted` | — | ❌ | RTF formatting details not needed for AI assistants; plain text is sufficient |
| 15 | `semantic_search` (document) | — | ⏳ Phase 3 | Requires vector embedding infrastructure |
| 16 | `find_analogies` | — | ⏳ Phase 3 | Requires HHM (Holographic Hyperdimensional Memory) |

## Search & Discovery

| # | dcondrey Tool | scrivener-mcp | Status | Notes |
|---|--------------|---------------|--------|-------|
| 17 | `search_content` | `search_content` | ✅ Phase 2 | Same functionality |
| 18 | `list_trash` | `list_trash` | ✅ Phase 2 | Same functionality |
| 19 | `search_trash` | `search_trash` | ✅ Phase 2 | Same functionality |
| 20 | `recover_document` | `recover_document` | ✅ Phase 2 | Same functionality |
| 21 | `get_document_annotations` | — | ⏳ Phase 3 | Requires RTF annotation parsing |
| 22 | `vector_search` | — | ⏳ Phase 3 | Requires vector embedding infrastructure |
| 23 | `find_mentions` | — | ⏳ Phase 3 | Entity recognition across documents |
| 24 | `cross_reference_analysis` | — | ⏳ Phase 3 | AI-powered cross-referencing |

## Compilation & Export

| # | dcondrey Tool | scrivener-mcp | Status | Notes |
|---|--------------|---------------|--------|-------|
| 25 | `compile_documents` | `compile_documents` | ✅ Phase 2 | Same functionality |
| 26 | `export_project` | `export_project` | ✅ Phase 2 | Same functionality |
| 27 | `get_statistics` | `get_statistics` | ✅ Phase 2 | Same functionality |
| 28 | `intelligent_compilation` | — | ⏳ Phase 3 | AI-powered compilation optimization |
| 29 | `generate_marketing_materials` | — | ❌ | AI content generation is client-side responsibility, not MCP tool scope |
| 30 | `build_vector_store` | — | ⏳ Phase 3 | Required for semantic search |

## Content Analysis & Enhancement

| # | dcondrey Tool | scrivener-mcp | Status | Notes |
|---|--------------|---------------|--------|-------|
| 31 | `analyze_document` | `analyze_document` | ✅ Phase 2 | Backed by writing-analysis crate (rule-based, no AI) |
| 32 | `enhance_content` | — | ❌ | AI content generation is client-side responsibility |
| 33 | `generate_content` | — | ❌ | AI content generation is client-side responsibility |
| 34 | `update_memory` | `update_memory` | ✅ Phase 2 | SQLite-backed instead of in-memory |
| 35 | `get_memory` | `get_memory` | ✅ Phase 2 | SQLite-backed instead of in-memory |
| 36 | `check_consistency` | `check_consistency` | ✅ Phase 2 | Rule-based comparison of memory vs documents |
| 37 | `multi_agent_analysis` | — | ❌ | Multi-agent orchestration is outside MCP server scope |
| 38 | `start_realtime_assistance` | — | ❌ | Real-time streaming not supported in stdio MCP |
| 39 | `collect_feedback` | — | ❌ | Feedback collection is a client-side concern |

## Asynchronous Job Operations

| # | dcondrey Tool | scrivener-mcp | Status | Notes |
|---|--------------|---------------|--------|-------|
| 40 | `queue_document_analysis` | — | ❌ | MCP tools are synchronous request-response; async queuing adds complexity without benefit |
| 41 | `queue_project_analysis` | — | ❌ | Same as above |
| 42 | `semantic_search` (async) | — | ⏳ Phase 3 | Covered by Phase 3 semantic search |
| 43 | `generate_ai_suggestions` | — | ❌ | AI generation is client-side |
| 44 | `analyze_writing_style` | `analyze_document` | ✅ Phase 2 | Covered by analyze_document with writing-analysis crate |
| 45 | `check_plot_consistency` | `check_consistency` | ✅ Phase 2 | Covered by check_consistency tool |
| 46 | `get_job_status` | — | ❌ | No async job queue |
| 47 | `cancel_job` | — | ❌ | No async job queue |
| 48 | `get_queue_stats` | — | ❌ | No async job queue |

## Fractal Memory System

| # | dcondrey Tool | scrivener-mcp | Status | Notes |
|---|--------------|---------------|--------|-------|
| 49 | `ingest_document_fractal` | — | ⏳ Phase 3 | Advanced memory system |
| 50 | `fractal_search` | — | ⏳ Phase 3 | Requires fractal memory infrastructure |
| 51 | `find_cooccurrences` | — | ⏳ Phase 3 | Entity co-occurrence analysis |
| 52 | `check_character_continuity` | — | ⏳ Phase 3 | Character tracking across documents |
| 53 | `track_motifs` | — | ⏳ Phase 3 | Motif/theme tracking |
| 54 | `ingest_project_fractal` | — | ⏳ Phase 3 | Batch ingestion |
| 55 | `update_retrieval_policy` | — | ⏳ Phase 3 | Custom retrieval policies |
| 56 | `get_memory_analytics` | — | ⏳ Phase 3 | Memory system analytics |
| 57 | `analyze_narrative` | — | ⏳ Phase 3 | Narrative structure analysis |
| 58 | `get_memory_stats` | — | ⏳ Phase 3 | Memory usage statistics |

## scrivener-mcp Only (not in dcondrey)

| # | Tool | Status | Notes |
|---|------|--------|-------|
| 59 | `analyze_readability` | ✅ Phase 2 | Dedicated readability tool (writing-analysis crate) |
| 60 | `get_project_summary` | ✅ Phase 2 | Combines project info + memory + stats |
| 61 | `get_writing_stats` | ✅ Phase 2 | Detailed per-document word count breakdown |
| 62 | `get_session_info` | ✅ Phase 2 | Server session information |

---

## Summary

| Status | Count | Description |
|--------|-------|-------------|
| ✅ Phase 2 | 29 | Core project operations, document CRUD, search, analysis, memory |
| ⏳ Phase 3 | 16 | Semantic search, vector store, fractal memory, entity tracking |
| ❌ Not planned | 14 | AI content generation, async queues, real-time streaming, multi-agent |
| **Total dcondrey** | **58** | |
| **scrivener-mcp only** | **5** | analyze_readability, create_folder, get_project_summary, get_writing_stats, get_session_info |

### Design Philosophy Differences

1. **No AI content generation in MCP tools**: dcondrey includes `enhance_content`, `generate_content`, `generate_ai_suggestions`, `generate_marketing_materials`. We exclude these because MCP clients (Claude, etc.) already have AI capabilities — the MCP server should provide *data access*, not *AI processing*.

2. **No async job queue**: dcondrey's async tools (`queue_*`, `get_job_status`, `cancel_job`) add infrastructure complexity. MCP tools are inherently request-response. If a tool takes too long, the solution is optimizing the tool, not adding a queue.

3. **Rule-based analysis**: dcondrey's analysis tools use external AI APIs. Our `analyze_document` uses the `writing-analysis` crate for deterministic, rule-based analysis (readability scores, passive voice detection, etc.) — fast, offline, and reproducible.

4. **SQLite memory vs in-memory**: dcondrey stores memory in-process. We use SQLite for persistence across sessions, with category-based organization.
