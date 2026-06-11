# rust-db

> **Status: In Progress** — actively being developed.

A custom database engine written in Rust, inspired by PostgreSQL and ClickHouse.

This is a personal learning project by [Sergio Kaznowiecki](https://github.com/skaznowiecki). The goal is to build a relational database from scratch — lexer, parser, storage engine, query execution — and see how far it can go. Built entirely in Rust as a way to learn the language through a real, complex project.

## Current Features

- **SQL Parser** — Hand-written lexer and parser (no dependencies) supporting:
  - `CREATE DATABASE`, `DROP DATABASE`, `SHOW DATABASES`, `SHOW TABLES`
  - `CREATE TABLE` with types and constraints, `DROP TABLE`
  - `INSERT INTO` with field and type validation
  - `SELECT` with specific columns or `*`, `WHERE` filtering (`=`, `!=`, `<`, `<=`, `>`, `>=`, `LIKE`, `ILIKE`, `IN`, `BETWEEN`), compound conditions (`AND`, `OR`, parentheses), and `LIMIT`
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
  - O(1) column index lookup via in-memory HashMap

- **Client-Server Architecture**
  - Multi-threaded TCP server on `localhost:5433` (one thread per connection)
  - Background flush thread (every 5 seconds)
  - Persistent connections (one TCP connection per REPL session)
  - Interactive REPL with persistent command history across sessions (`db connect`)
  - Single-command mode (`db exec`)
  - Background server with `db start` / `db stop` / `db info`

- **Query Timing** — Displays execution time after each query result

- **Custom Error Types** — Typed `DbError` enum instead of string errors

## Example

```bash
./target/release/db start
./target/release/db connect
```

```sql
SHOW DATABASES;
-- +-----------+
-- | Database  |
-- +-----------+
-- | ecommerce |
-- +-----------+
-- (1 rows)

CREATE DATABASE ecommerce;
USE ecommerce;

CREATE TABLE productos (
  id SERIAL PRIMARY KEY,
  nombre VARCHAR(200) NOT NULL,
  precio INTEGER NOT NULL,
  stock INTEGER DEFAULT 0,
  activo BOOLEAN DEFAULT TRUE
);

SHOW TABLES;
-- +-----------+
-- | Table     |
-- +-----------+
-- | productos |
-- +-----------+
-- (1 rows)

INSERT INTO productos (nombre, precio, stock) VALUES ('Laptop Samsung', 15000, 50);
INSERT INTO productos (nombre, precio, stock) VALUES ('Mouse Logitech', 2500, 200);
INSERT INTO productos (nombre, precio, stock, activo) VALUES ('Teclado HP', 4500, 0, FALSE);
INSERT INTO productos (nombre, precio, stock) VALUES ('Monitor LG', 35000, 15);
INSERT INTO productos (nombre, precio, stock) VALUES ('Auriculares Sony', 8900, 80);

SELECT * FROM productos;
-- +----+-------------------+--------+-------+--------+
-- | id | nombre            | precio | stock | activo |
-- +----+-------------------+--------+-------+--------+
-- | 1  | Laptop Samsung    | 15000  | 50    | true   |
-- | 2  | Mouse Logitech    | 2500   | 200   | true   |
-- | 3  | Teclado HP        | 4500   | 0     | false  |
-- | 4  | Monitor LG        | 35000  | 15    | true   |
-- | 5  | Auriculares Sony  | 8900   | 80    | true   |
-- +----+-------------------+--------+-------+--------+
-- (5 rows)

SELECT * FROM productos WHERE precio = 2500;
-- +----+----------------+--------+-------+--------+
-- | id | nombre         | precio | stock | activo |
-- +----+----------------+--------+-------+--------+
-- | 2  | Mouse Logitech | 2500   | 200   | true   |
-- +----+----------------+--------+-------+--------+
-- (1 rows)
-- Time: 0.045ms

SELECT * FROM productos WHERE precio > 5000;
SELECT * FROM productos WHERE precio >= 2500 AND precio <= 15000;
SELECT * FROM productos WHERE nombre != 'Teclado HP';
SELECT * FROM productos WHERE nombre LIKE '%Sony%';
SELECT * FROM productos WHERE nombre ILIKE '%samsung%';
SELECT * FROM productos WHERE nombre IN ('Laptop Samsung', 'Monitor LG');
SELECT * FROM productos WHERE precio BETWEEN 2000 AND 10000;
SELECT nombre, precio FROM productos WHERE precio > 5000;
SELECT * FROM productos WHERE (precio > 10000 OR stock = 0) AND activo = TRUE;

SELECT * FROM productos LIMIT 2;
-- +----+----------------+--------+-------+--------+
-- | id | nombre         | precio | stock | activo |
-- +----+----------------+--------+-------+--------+
-- | 1  | Laptop Samsung | 15000  | 50    | true   |
-- | 2  | Mouse Logitech | 2500   | 200   | true   |
-- +----+----------------+--------+-------+--------+
-- (2 rows)

DROP TABLE productos;
DROP DATABASE ecommerce;
```

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

# Interactive REPL (with command history via arrow keys)
./target/release/db connect

# Single command
./target/release/db exec "SHOW DATABASES"
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

### Connect to the REPL

```bash
# 1. Build (first time, or after code changes)
cargo build --release

# 2. Optional: start the server in background
./target/release/db start

# 3. Open the interactive REPL
./target/release/db connect
```

If the server is running, `connect` attaches to it on `localhost:5433`. If not, it falls back to local mode automatically. During development you can also use `cargo run -- connect` (debug build, no release step).

After rebuilding, restart the server so `connect` and `exec` pick up your changes (`db stop` then `db start`). If `db stop` reports no PID file but queries still fail with old behavior, an orphaned process may still be listening on port 5433 — stop it with `kill $(lsof -i :5433 -t)` and start again.

Exit the REPL with `Ctrl+D` or `\q`.

### Development with cargo-watch

Install once:

```bash
cargo install cargo-watch
```

Useful workflows:

```bash
# Rebuild on every save (use connect in a second terminal)
cargo watch -x build

# Run tests on every save
cargo watch -x test

# Re-run a SQL statement on every save
cargo watch -x 'run -- exec "SELECT 1"'
cargo watch -x 'run -- exec --db myapp "SELECT * FROM users"'

# Rebuild and restart the server on every save
cargo watch -c -x 'build --release' -s './target/release/db stop 2>/dev/null; ./target/release/db start'
```

`connect` is interactive, so it does not pair well with `cargo watch` relaunching it on every save. Prefer `cargo watch -x build` in one terminal and `./target/release/db connect` (or `cargo run -- connect`) in another.

### Run tests

```bash
cargo test
```

## Architecture

```
src/
├── main.rs                  CLI entry point
├── lib.rs
├── constants.rs             Paths, buffer size, port
├── error.rs                 DbError enum
├── engine/
│   ├── engine.rs            Engine (orchestration, column projection, table formatting)
│   ├── database.rs          Database (manages tables, catalog)
│   └── table/
│       ├── mod.rs            Table struct, construction, flush, buffer
│       ├── insert.rs         Validation, row building, insert, serial counter
│       ├── scan.rs           Full table scan with WHERE and LIMIT
│       └── filter.rs         WHERE evaluation (operators, IN, BETWEEN, AND/OR)
├── lexer/
│   ├── token.rs             Token enum
│   └── lexer.rs             Tokenizer
├── parser/
│   ├── ast.rs               AST types (Statement, Value, DataType, WhereExpr, etc.)
│   ├── parser.rs            Parser core + dispatch
│   └── statements/          One file per SQL statement (select, insert_into, etc.)
├── provider/
│   ├── server.rs            TCP server
│   ├── client.rs            TCP client (persistent connections)
│   ├── repl.rs              Interactive REPL with history (rustyline)
│   └── command.rs           Single command (local/remote)
└── storage/
    ├── schema.rs             JSON schema load/save
    ├── file_utils.rs         read_last_line (seek from end)
    ├── catalog.rs            Catalog column definitions
    └── statements/           One file per storage operation
```

## Roadmap

### Phase 1 — Query Engine
| Feature | Status | Description |
|---|---|---|
| `SHOW DATABASES` | Done | List all databases (directories with `schema.json` under `./data/`) |
| `SHOW TABLES` | Done | List all tables in the current database (from the catalog) |
| `SELECT` | Done | Column projection, `WHERE` (all comparison operators, `LIKE`/`ILIKE`, `IN`, `BETWEEN`, `AND`/`OR`, parentheses), `LIMIT` |
| `ORDER BY` | Pending | Sort results by one or more columns (`ASC`/`DESC`) |
| Aggregations | Pending | `COUNT`, `SUM`, `AVG`, `MIN`, `MAX` |
| `GROUP BY` | Pending | Group results by column with aggregation functions |
| `HAVING` | Pending | Filter groups after aggregation |
| `UPDATE` | Pending | Modify existing rows |
| `DELETE` | Pending | Remove rows |

### Phase 2 — Storage Engine
| Feature | Description |
|---|---|
| Binary format | Page-based storage replacing pipe-delimited text |
| B-tree indexes | Indexed lookups for faster queries |
| WAL | Write-ahead log for crash recovery |

### Phase 3 — Advanced
| Feature | Description |
|---|---|
| Query planner | Cost-based optimization for query execution |
| `JOIN` support | Cross-table queries |
| Aggregations | `COUNT`, `SUM`, `AVG`, `GROUP BY` |
