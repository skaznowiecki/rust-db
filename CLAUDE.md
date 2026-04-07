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

## Catalog Pattern (Storage)

Every database has a catalog table (ID 1000) that tracks which tables exist and their IDs. Table schemas are stored as JSON files. This is how the engine discovers tables on startup without hardcoding anything.

## Buffered Writes

Tables accumulate writes in an in-memory buffer (200KB). The buffer flushes to disk when full, on a timer (5 seconds), or when the table is dropped from memory. This reduces disk I/O for bulk inserts.

## Client-Server with Local Fallback

The REPL (`db connect`) tries to connect to a running server first. If no server is running, it falls back to local mode (direct engine access). Both modes share the same REPL code with different backends.

## Conventions

- **Text storage**: Row data is pipe-delimited (`|`), one row per line. Simple, readable, grep-friendly.
- **Numeric IDs for tables**: Tables get auto-incrementing IDs (starting at 1001). Directories are named by ID, not table name. This avoids filesystem issues with special characters.
- **Error handling**: All errors use the `DbError` enum — no `.unwrap()` in business logic, no string errors.
- **No external DB dependencies**: The lexer, parser, and engine are hand-written. Only external crates are `rustyline` (REPL), `serde` (JSON), and `libc` (process management).
