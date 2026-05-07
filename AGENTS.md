# Zinq Agent Instructions

## Rust Edition

Uses `edition = "2024"` (unstable). Requires nightly or recent stable Rust (1.90+ confirmed).

## Dev Commands

```bash
# Run the server (requires ScyllaDB running)
cargo run

# Run tests (spins up a local ScyllaDB via testcontainers)
cargo run --bin test

# DB schema management (requires ScyllaDB at 127.0.0.1:9042)
cargo run --bin db -- init
cargo run --bin db -- reset
cargo run --bin db -- recreate-table <TABLE_NAME>
```

## Database

- **ScyllaDB** (Cassandra-compatible). Start with `docker compose up -d` or the `db init` command.
- Schema lives in `migrations/1.cql`. All tests read this file and create a test keyspace per test.
- Tests require env vars `TEST_SCYLLA_HOST` and `TEST_SCYLLA_PORT` — the `scripts/test.rs` runner sets these automatically via testcontainers.

## Auth / JWT

- JWT uses **RSA keys** loaded from the `keys/` directory.
- `keys/` is gitignored. The server **panics on startup** if keys are missing.
- The `keys/key.pem` file exists in the repo and is required for runtime.
- Tests load keys from `"keys"` (relative to working directory at repo root).

## Config

- `.env` contains `PORT` and `SCYLLA_NODE`. All other config (SMTP, JWT, keys dir) has sensible defaults or reads from env vars.
- Config is loaded via `dotenvy` at startup (`src/config.rs`).

## Architecture

Single Rust crate. Key modules:
- `src/domain/` — entities, repository traits, event bus
- `src/application/` — command/query handlers (CQRS-style)
- `src/infra/` — repository implementations (Scylla), auth services, SMTP
- `src/gateway/` — socket.io WebSocket layer
- `src/routers/` — HTTP route modules

## Important Notes

- Tests are in `src/tests/` and run against a real ScyllaDB instance (no mocking).
- `src/tests/common/test_context.rs` creates a fresh keyspace per test context by re-executing the migration file.
- The `db` binary uses `clap` CLI with `init`, `reset`, and `recreate-table` subcommands.
