# Contributing to TechTown Payroll Backend

Thank you for helping improve TechTown Payroll Backend. This project is a Rust
API service for DAO payroll workflows with PostgreSQL, Redis, and Stellar RPC
dependencies.

## Local Development Setup

### 1. Install prerequisites

You need:

- Rust stable with Cargo
- PostgreSQL 14 or newer
- Redis 6 or newer
- A Soroban-compatible Stellar RPC endpoint
- `sqlx-cli` for database migrations

Install the SQLx CLI:

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### 2. Fork and clone

```bash
git clone https://github.com/YOUR_USERNAME/techtown-payroll-backend.git
cd techtown-payroll-backend
git remote add upstream https://github.com/TechTown-Private-DAO/techtown-payroll-backend.git
```

Before starting new work, sync with upstream:

```bash
git checkout main
git pull upstream main
```

### 3. Start local services

Run PostgreSQL and create a local database:

```bash
createdb techtown_payroll
```

Run Redis locally:

```bash
redis-server
```

For Stellar RPC, use a Soroban-compatible testnet endpoint during local
development, for example:

```text
https://soroban-testnet.stellar.org
```

### 4. Configure environment variables

Copy the sample environment file:

```bash
cp .env.example .env
```

Then update `.env` for your machine.

| Variable | Purpose |
| --- | --- |
| `DATABASE_URL` | PostgreSQL connection string used by the API and SQLx migrations. |
| `REDIS_URL` | Redis connection string used for payroll caching. |
| `STELLAR_RPC_URL` | Soroban-compatible Stellar RPC endpoint for on-chain calls. |
| `STELLAR_NETWORK_PASSPHRASE` | Stellar network passphrase matching the RPC endpoint. |
| `JWT_SECRET` | Secret used to sign authentication tokens. Use a long local-only value. |
| `JWT_EXPIRATION` | Token lifetime in seconds. The sample uses `3600`. |
| `PORT` | API server port. The sample uses `3000`. |
| `CORS_ORIGIN` | Frontend origin allowed by CORS, such as `http://localhost:3001`. |

Never commit real secrets, funded wallet keys, production database URLs, or
private RPC credentials.

### 5. Run migrations

Make sure `DATABASE_URL` points at your local database, then run:

```bash
sqlx migrate run
```

### 6. Build, test, and run

Build the project:

```bash
cargo build --locked
```

Run tests the same way CI does:

```bash
SQLX_OFFLINE=true cargo test --locked
```

Run the API locally:

```bash
cargo run
```

The server listens on the port configured by `PORT`.

## Making Changes

Create a focused branch:

```bash
git checkout -b docs/update-contributing-guide
```

Use a branch prefix that matches the change type:

- `docs/` for documentation
- `fix/` for bug fixes
- `feat/` for new behavior
- `test/` for test-only changes
- `chore/` for maintenance

Keep pull requests small and tied to one issue when possible.

## Code Style

Before opening a pull request, run:

```bash
cargo fmt
cargo clippy --all-targets --all-features
SQLX_OFFLINE=true cargo test --locked
```

Use Rust's standard formatting. Prefer clear names, explicit error handling, and
small functions that keep API, database, and Stellar RPC responsibilities easy to
review.

## Commit Messages

Use short, descriptive commit messages. Conventional-style prefixes are welcome:

```text
docs: add local setup notes
fix: handle missing payroll cache entry
test: cover payroll approval flow
```

## Pull Request Checklist

Before requesting review:

- [ ] The branch is up to date with `main`.
- [ ] The change is scoped to one issue or one clear improvement.
- [ ] Environment variables or secrets are not committed.
- [ ] Database migrations are included when schema changes are made.
- [ ] `cargo fmt` has been run.
- [ ] `cargo clippy --all-targets --all-features` has been run when code changes are made.
- [ ] `SQLX_OFFLINE=true cargo test --locked` passes.
- [ ] README or docs are updated when behavior or setup changes.

In the pull request description, include:

- the issue being addressed;
- a short summary of the change;
- the checks you ran locally;
- any setup notes reviewers need to reproduce the change.

## Reviews

Maintainers may ask for smaller diffs, extra tests, migration updates, or clearer
setup notes. Please respond in the pull request thread and keep follow-up commits
focused on the requested change.
