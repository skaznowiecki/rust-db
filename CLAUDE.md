# Project Instructions

- Always update README.md when adding new features or changing existing behavior.

# Project Purpose

rust-db is a relational database engine built from scratch in Rust as a learning project. The goal is to understand how databases work internally — from SQL parsing to storage — by implementing every layer without relying on external database libraries.

# Architecture & Design Patterns

## Pipeline Pattern (SQL Processing)

Every SQL statement flows through a strict 3-stage pipeline:

```
SQL string → Lexer → Tokens → Parser → AST → Engine → Result
```

Each stage has a single responsibility and communicates only through its output type. No stage knows about the internals of another.

## Single-File-Per-Statement (Parser)

Each SQL statement (SELECT, INSERT, CREATE TABLE, etc.) has its own file under `src/parser/statements/`. The main parser dispatches to the correct file based on the first keyword. This keeps each parser small and focused.

## Single-Responsibility Modules (Table)

The table logic is split into focused modules under `src/engine/table/`:
- `mod.rs` — struct definition, construction, flush, Drop
- `insert.rs` — validation, row building, insert, serial counter
- `scan.rs` — full table scan with WHERE and LIMIT
- `filter.rs` — WHERE evaluation: operators (=, !=, <, <=, >, >=, LIKE, ILIKE), IN, BETWEEN, and recursive AND/OR with parentheses via `WhereExpr` tree

New table operations (DELETE, UPDATE, aggregations) get their own file. New WHERE operators go in `filter.rs`.

## Catalog Pattern (Storage)

Every database has a catalog table (ID 1000) that tracks which tables exist and their IDs. Table schemas are stored as JSON files. This is how the engine discovers tables on startup without hardcoding anything.

## Buffered Writes

Tables accumulate writes in an in-memory buffer (200KB). The buffer flushes to disk when full, on a timer (5 seconds), or when the table is dropped from memory. This reduces disk I/O for bulk inserts.

## Client-Server with Local Fallback

The REPL (`db connect`) tries to connect to a running server first. If no server is running, it falls back to local mode (direct engine access). Both modes share the same REPL code with different backends.

## Conventions

- **Text storage**: Row data is pipe-delimited (`|`), one row per line. Simple, readable, grep-friendly.
- **Numeric IDs for tables**: Tables get auto-incrementing IDs (starting at 1001). Directories are named by ID, not table name. This avoids filesystem issues with special characters.
- **Error handling**: Surface failures with the `DbError` enum; avoid ad-hoc string errors. Prefer `Result` and `?` in the engine; do not `unwrap` on paths that should return a user-facing error. Reserve `expect`/`unwrap` only for bugs or OS/bootstrap code where recovery is not meaningful.
- **No external DB dependencies**: The lexer, parser, and engine are hand-written. External crates: `rustyline` (REPL), `serde` + `serde_json` (schema and catalog JSON on disk), and `libc` (process management). Do not add database driver crates for the engine core.
