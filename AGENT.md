# Agent context — rust-db

Use this file with `CLAUDE.md` (project source of truth). This summary is tailored for AI assistants editing this repository.

## What this repo is

**rust-db** is a relational database engine implemented from scratch in **Rust** as a learning project. The goal is to implement every layer (SQL through storage) **without** external database libraries.

## When you change code

- **Update `README.md`** whenever you add features or change documented behavior. This is a project rule.

## SQL processing (non-negotiable shape)

Data flows in one direction through a **strict pipeline**; stages only talk via their output types:

```text
SQL string → Lexer → Tokens → Parser → AST → Engine → Result
```

Do not couple lexer/parser/engine internals across stages.

## Where to put new code

| Area | Location | Rule |
|------|----------|------|
| **Parser — per statement** | `src/parser/statements/` | One file per statement type (SELECT, INSERT, CREATE TABLE, …). Main parser dispatches on the first keyword. |
| **Table — behavior** | `src/engine/table/` | Split by concern: `mod.rs` (struct, build, flush, `Drop`), `insert.rs` (validation, row build, insert, serial), `scan.rs` (full scan + WHERE + LIMIT), `filter.rs` (WHERE: `= != < <= > >= LIKE ILIKE`, IN, BETWEEN, AND/OR + parens via `WhereExpr`). New table ops (DELETE, UPDATE, aggs) → **new file**. New WHERE operators → **`filter.rs`**. |
| **Storage / catalog** | Catalog is table **ID 1000**; table schemas on disk as **JSON**. Engine discovers tables on startup from this—no hardcoded table lists. |

## Runtime behavior to respect

- **Buffered writes**: In-memory buffer (~**200KB**). Flush when full, on a **~5s** timer, or when the table is dropped from memory.
- **REPL** (`db connect`): Prefer a **running server**; if none, use **local** mode (direct engine). Same REPL, different backends.

## Conventions

- **Row text**: Pipe-delimited (`|`), one row per line.
- **Table directories**: **Numeric IDs** (auto-increment from **1001**), not names—avoids filesystem issues with special characters.
- **Errors**: Prefer **`DbError`** and `Result`; no ad-hoc string errors. Use `?` on engine paths; only `unwrap`/`expect` where invariants are guaranteed or in bootstrap/CLI (see `CLAUDE.md`).
- **Dependencies**: No external DB stack. Allowed crates: **`rustyline`** (REPL), **`serde`** + **`serde_json`** (on-disk JSON schemas/catalog), **`libc`** (process management). No database driver crates for the engine core.

## Quick checks before a PR or handoff

- README reflects behavior changes.
- New SQL statements or table features land in the files/modules above, not in monolithic catch-alls.
- Errors stay typed (`DbError`).
