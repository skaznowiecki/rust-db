# rust-db

> **Status: In Progress** — actively being developed.

A custom database engine written in Rust, inspired by PostgreSQL and ClickHouse.

This is a personal learning project by [Sergio Kaznowiecki](https://github.com/skaznowiecki). The goal is to build a relational database from scratch — lexer, parser, storage engine, query execution — and see how far it can go. Built entirely in Rust as a way to learn the language through a real, complex project.

## Current Features

- **SQL Parser** — Hand-written lexer and parser (no dependencies) supporting:
  - `CREATE DATABASE`, `DROP DATABASE`
  - `CREATE TABLE` with types and constraints, `DROP TABLE`
  - `INSERT INTO` with field and type validation
  - `SELECT * FROM` with `WHERE` filtering (`=`, `!=`, `<`, `>`, `LIKE`, `ILIKE`) and `LIMIT`
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
CREATE DATABASE ecommerce;
USE ecommerce;

CREATE TABLE productos (
  id SERIAL PRIMARY KEY,
  nombre VARCHAR(200) NOT NULL,
  precio INTEGER NOT NULL,
  stock INTEGER DEFAULT 0,
  activo BOOLEAN DEFAULT TRUE
);

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
SELECT * FROM productos WHERE nombre != 'Teclado HP';
SELECT * FROM productos WHERE nombre LIKE '%Sony%';
SELECT * FROM productos WHERE nombre ILIKE '%samsung%';

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

## Architecture

```
src/
├── main.rs                  CLI entry point
├── lib.rs
├── constants.rs             Paths, buffer size, port
├── error.rs                 DbError enum
├── engine/
│   ├── engine.rs            Engine (orchestration, table formatting)
│   ├── database.rs          Database (manages tables, catalog)
│   └── table.rs             Table (schema, serial, validation, insert, scan, buffer)
├── lexer/
│   ├── token.rs             Token enum
│   └── lexer.rs             Tokenizer
├── parser/
│   ├── ast.rs               AST types (Statement, Value, DataType, WhereClause, etc.)
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
| `SELECT` | Done | Queries with `WHERE` (`=`, `!=`, `<`, `>`, `LIKE`, `ILIKE`) and `LIMIT` |
| `SELECT` operators | Pending | `>=`, `<=`, `AND`, `OR`, `IN`, `BETWEEN` |
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
