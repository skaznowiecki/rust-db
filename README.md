# db

A custom database engine written in Rust, inspired by PostgreSQL and ClickHouse.

This is a personal learning project by [Sergio Kaznowiecki](https://github.com/skaznowiecki). The goal is to build a relational database from scratch — lexer, parser, storage engine, query execution — and see how far it can go. Built entirely in Rust as a way to learn the language through a real, complex project.

## Current Features

- **SQL Parser** — Hand-written lexer and parser (no dependencies) supporting:
  - `CREATE DATABASE`, `DROP DATABASE`
  - `CREATE TABLE` with types and constraints, `DROP TABLE`
  - `INSERT INTO` with field and type validation
  - `USE` to switch databases

- **Data Types** — `SERIAL`, `INTEGER`, `VARCHAR(n)`, `TEXT`, `BOOLEAN`

- **Constraints** — `PRIMARY KEY`, `NOT NULL`, `UNIQUE`, `DEFAULT`

- **Storage Engine**
  - One directory per database, one subdirectory per table (identified by numeric ID)
  - Internal catalog table (ID 1000) that tracks all tables
  - Schema stored as JSON per database
  - Row data stored as pipe-delimited text files (one line per row)
  - SERIAL auto-increment with in-memory counter (reads disk only on startup)
  - 200KB write buffer per table, flushed periodically or on shutdown

- **Client-Server Architecture**
  - TCP server on `localhost:5433`
  - Persistent connections (one TCP connection per REPL session)
  - Interactive REPL (`db connect`) and single-command mode (`db exec`)
  - Background server with `db start` / `db stop` / `db info`

- **Custom Error Types** — Typed `DbError` enum instead of string errors

## Getting Started

### Prerequisites

- [Rust](https://rustup.rs/) (edition 2024)

### Build

```bash
cargo build --release
```

### Run

```bash
# Start the server (background)
./target/release/db start

# Interactive REPL
./target/release/db connect

# Single command
./target/release/db exec "CREATE DATABASE myapp"
./target/release/db exec --db myapp "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(100) NOT NULL)"
./target/release/db exec --db myapp "INSERT INTO users (name) VALUES ('John')"

# Stop the server
./target/release/db stop

# Check server status
./target/release/db info
```

### Run without server (local mode)

If no server is running, `connect` and `exec` work in local mode automatically.

### Run tests

```bash
cargo test
```

### Benchmark script

```bash
./scripts/setup.sh
```

Creates a database with 100k rows and measures insert performance.

## Architecture

```
src/
├── main.rs                  CLI entry point
├── lib.rs
├── error.rs                 DbError enum
├── engine/
│   ├── engine.rs            Engine (orchestration)
│   ├── database.rs          Database (manages tables, catalog)
│   └── table.rs             Table (schema, serial, validation, insert, buffer)
├── lexer/
│   ├── token.rs             Token enum
│   └── lexer.rs             Tokenizer
├── parser/
│   ├── ast.rs               AST types (Statement, Value, DataType, etc.)
│   ├── parser.rs            Parser core + dispatch
│   └── statements/          One file per SQL statement
├── provider/
│   ├── server.rs            TCP server
│   ├── client.rs            TCP client (persistent connections)
│   ├── repl.rs              Interactive REPL (local/remote)
│   └── command.rs           Single command (local/remote)
└── storage/
    ├── schema.rs             JSON schema load/save
    ├── file_utils.rs         read_last_line (seek from end)
    ├── catalog.rs            Catalog column definitions
    ├── constants.rs          DATA_DIR, CATALOG_ID, etc.
    └── statements/           One file per storage operation
```

## Next Up

- [ ] Multi-threaded server (concurrent client connections)
- [ ] Background flush thread (periodic write buffer flush)
- [ ] Catalog cache in memory (avoid disk reads for table name lookups)
- [ ] `SELECT` with basic `WHERE` filtering
- [ ] `UPDATE` and `DELETE` statements
- [ ] Binary storage format (pages)
- [ ] Indexes (B-tree)
- [ ] Query planner
