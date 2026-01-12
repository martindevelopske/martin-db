# Pesapal Junior Dev Challenge : Rust RDBMS

## Overview

This project is a custom-built Relational Database Management System (RDBMS) implemented from scratch in **Rust**. It features a hand-written SQL parser, an in-memory execution engine with persistent JSON storage, O(1) indexing for constraints, and a multi-interface design (REPL and Web).

## Technical Architecture

The system is architected as a core library (`martin-db`) with two separate binary front-ends. This modularity ensures the database logic is decoupled from the user interface.

- **Storage Engine**: Data is persisted as structured JSON. To ensure performance and integrity, indexes are **rebuilt in-memory** upon startup, avoiding "stale index" bugs and keeping the storage footprint small.
- **SQL Parser**: Instead of using Regular Expressions or third-party parser generators, I implemented a **Recursive Descent Parser**. This allows for more robust error reporting and easier extension of the SQL dialect.
- **Execution Engine**: Implements the core CRUD logic.
  - **Indexing**: Uses `HashSets` to provide $O(1)$ time complexity for checking `PRIMARY KEY` and `UNIQUE` constraints during insertion.
  - **Joins**: Implements a **Nested Loop Join** algorithm to combine data from multiple tables.
- **Concurrency**: The Web App uses an `Arc<RwLock<Database>>` pattern to allow safe, concurrent access to the engine across multiple HTTP threads.

## Features

- **Data Types**: Supports `INT` and `TEXT`.
- **Constraints**: Enforces `PRIMARY KEY` (must be unique and non-null) and `UNIQUE`.
- **Joins**: Supports joining two tables via the `JOIN ... ON ... = ...` syntax.
- **REPL**: A professional-grade CLI with command history and tab completion.
- **Web App**: A trivial dashboard to visualize table joins and perform live inserts.

## Tech Stack

- **Language**: Rust (2021 Edition)
- **Web Framework**: Axum / Tokio
- **CLI**: Rustyline (for the REPL experience)
- **Serialization**: Serde / Serde_JSON

## Getting Started

### Prerequisites

- [Rust & Cargo](https://rustup.rs/) (latest stable)

### Running the REPL

The REPL is the primary way to interact with the database engine.

```bash
cargo run --bin repl
```

**Example Commands:**

```sql
CREATE TABLE teams (id INT PRIMARY, name TEXT UNIQUE)
INSERT INTO teams VALUES (1, 'Engineering')
CREATE TABLE devs (id INT PRIMARY, name TEXT, team_id INT)
INSERT INTO devs VALUES (101, 'Alice', 1)
SELECT * FROM devs JOIN teams ON team_id = id
```

#### Note: for now, the queries do not support ending with a semi-colon(;), do not include it in the query while testing.

### Running the Web App

The web app demonstrates the RDBMS's capability to serve as a backend for applications.

```bash
cargo run --bin web_app
```

Then navigate to `http://127.0.0.1:3000`.

## Design Decisions & Ingenuity

1. **Manual Index Reconstruction**: I chose to skip serializing indexes to disk. By reconstructing the `HashSet` from raw data on load, the system guarantees that the index is always a perfect reflection of the data, even if the JSON file was manually edited.
2. **Recursive Descent Parser**: By building the tokenizer and parser manually, I gained full control over the syntax, allowing for clearer error messages (e.g., "Expected TABLE after CREATE").
3. **Safety over Speed**: Leveraging Rust’s `Result` and `Option` types, the system is designed to be "crash-proof" against malformed SQL queries.

## AI Disclosure & Credits

This project was developed with the assistance of AI for architectural guidance and boilerplate generation for the Web UI and the README file and also better functions documentation for my impentations . The core execution engine, join algorithms, and Rust-specific memory management patterns were refined and implemented by me to meet the specific requirements of the Pesapal challenge.

---

**Author:** [Martin Ndung'u]  
**Submission Date:** January 12, 2026
