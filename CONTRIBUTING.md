# Contributing to TechTown Payroll Backend

Welcome! This guide will walk you through setting up the project locally and getting your first PR merged.

## Table of Contents

- [Prerequisites](#prerequisites)
- [1. Clone and Install Dependencies](#1-clone-and-install-dependencies)
- [2. Start Local Services](#2-start-local-services)
- [3. Configure Environment](#3-configure-environment)
- [4. Run Migrations](#4-run-migrations)
- [5. Build and Run](#5-build-and-run)
- [6. Run Tests and Linters](#6-run-tests-and-linters)
- [Environment Variable Reference](#environment-variable-reference)
- [Commit and PR Conventions](#commit-and-pr-conventions)
- [Appendix: sqlx Offline Mode](#appendix-sqlx-offline-mode)

---

## Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Rust | 1.75+ | Install via [rustup](https://rustup.rs/) |
| PostgreSQL | 14+ | Local instance or Docker container |
| Redis | 6+ | Local instance or Docker container |
| Stellar RPC | Soroban-compatible | Soroban testnet RPC is fine for local dev |

> **Tip:** The CI pipeline uses `cargo build --locked` and `cargo test --locked`. Always use `--locked` when building in CI contexts.

---

## 1. Clone and Install Dependencies

```bash
git clone https://github.com/<your-org>/techtown-payroll-backend.git
cd techtown-payroll-backend

# Install Rust tools
rustup show  # confirms Rust version
cargo --version # should be >= 1.75

# Install sqlx-cli (needed for migrations)
cargo install sqlx-cli
```

Optionally install `cargo-watch` for auto-reload during development:

```bash
cargo install cargo-watch
```

---

## 2. Start Local Services

You need three services running locally: **PostgreSQL**, **Redis**, and a **Stellar Soroban RPC** endpoint.

### Option A: Docker (recommended)

```bash
# Start PostgreSQL and Redis
docker run --name techtown-postgres \
  -e POSTGRES_USER=user \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=techtown_payroll \
  -p 5432:5432 \
  -d postgres:15

docker run --name techtown-redis \
  -p 6379:6379 \
  -d redis:7-alpine
```

For the Stellar RPC, you can use the public testnet endpoint:

```
https://soroban-testnet.stellar.org
```

No local container is required for basic development — the public testnet RPC is sufficient for running and testing most features. If you prefer running a local RPC, see the [Stellar docs](https://soroban.io/).

### Option B: Native install

- **PostgreSQL:** Install via your OS package manager. Create a database named `techtown_payroll` and a user with credentials matching your `.env`.
- **Redis:** Install via your OS package manager. The default binding is `localhost:6379`.

---

## 3. Configure Environment

Copy `.env.example` to `.env` in the project root and fill in values:

```bash
cp .env.example .env
```

Edit `.env` with your local settings. Every variable is explained in the [Environment Variable Reference](#environment-variable-reference) section below.

> **Windows users:** Use PowerShell or Git Bash instead of `cp`.
>
> `.env` is git-ignored. Never commit `.env` to version control.

---

## 4. Run Migrations

Migrations live in `migrations/` and are written in raw SQL.

```bash
sqlx migrate run
```

This applies any pending migrations to the database pointed at by `DATABASE_URL`. The server also automatically runs migrations on startup via `src/db/mod.rs`, so running `sqlx migrate run` is optional if you plan to start the server immediately.

---

## 5. Build and Run

```bash
# Build
cargo build

# Run the server
cargo run
```

The server listens on `http://localhost:3000` (or whatever `PORT` is set to in `.env`).

For development with auto-reload on file changes:

```bash
cargo watch -x run
```

---

## 6. Run Tests and Linters

```bash
# Unit / integration tests
cargo test

# Lint (compile with warnings as errors)
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

### CI parity

The CI pipeline (`.github/workflows/ci.yml`) runs:

```bash
cargo build --locked
cargo test --locked
```

It sets `SQLX_OFFLINE=true`, which means it compiles against the checked-in `.sqlx/` query cache and does not require a live database.

---

## Environment Variable Reference

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `DATABASE_URL` | Yes | — | PostgreSQL connection string. Format: `postgres://USER:PASSWORD@HOST:PORT/DATABASE`. Example: `postgres://user:password@localhost:5432/techtown_payroll` |
| `REDIS_URL` | Yes | — | Redis connection string. Format: `redis://HOST:PORT`. Example: `redis://localhost:6379` |
| `STELLAR_RPC_URL` | Yes | — | Soroban-compatible RPC endpoint. Use `https://soroban-testnet.stellar.org` for testnet or a mainnet RPC for production. Example: `https://soroban-testnet.stellar.org` |
| `STELLAR_NETWORK_PASSPHRASE` | Yes | — | Human-readable network identifier used for transaction signing. **Testnet:** `Test SDF Network ; September 2015`. **Mainnet:** `Public Global Stellar Network ; September 2015` |
| `JWT_SECRET` | Yes | — | Secret key used to sign and verify JWTs. Must be a long, random string. Never use the example value in production. Generate with `openssl rand -hex 64` or a similar tool. |
| `JWT_EXPIRATION` | Yes | `3600` | JWT expiry time in seconds. `3600` = 1 hour. |
| `PORT` | No | `3000` | TCP port the API server binds to. |
| `CORS_ORIGIN` | No | `*` | Allowed origin for browser-based clients. Set to your frontend URL (e.g., `http://localhost:3001`). Note: the current code sets CORS to allow any origin (`Any`); this config value is currently not applied to the CORS layer. |

---

## Commit and PR Conventions

### Commit Messages

We follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.1.0/):

```
<type>(<scope>): <subject>
```

**Types:**
- `feat` — a new feature
- `fix` — a bug fix
- `docs` — documentation changes
- `refactor` — code change that neither fixes a bug nor adds a feature
- `test` — adding or updating tests
- `chore` — maintenance (CI, build, dependencies)

**Scope (optional):** Use the module or affected area, e.g., `feat(auth)`, `fix(db)`.

**Examples:**
```
feat(auth): add refresh token endpoint
fix(db): handle null stellar_rpc_url gracefully
docs: add local Postgres setup instructions
```

### Pull Requests

1. **Branch from `main`** (or `master`).
2. **Keep PRs small.** One feature or fix per PR.
3. **Update docs** if you change behavior. Update `README.md`, `NEXT_STEPS.md`, or add comments where appropriate.
4. **Ensure these pass locally before pushing:**
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   cargo test
   cargo build --locked
   ```
5. **Reference related issues** in the PR description using `Closes #123` or `Related to #456`.
6. **Describe the change clearly** in the PR description. State what changed, why, and how to verify.

---

## Appendix: sqlx Offline Mode

This project commits pre-compiled query results in `.sqlx/` to speed up builds without a database.

When you add or modify SQL queries in the codebase, you must regenerate the offline data:

```bash
# With a live DATABASE_URL set
cargo sqlx prepare
```

Then commit the updated `.sqlx/` files alongside your code changes.

If you only change Rust (non-SQL) code, you do not need to run `cargo sqlx prepare`.
