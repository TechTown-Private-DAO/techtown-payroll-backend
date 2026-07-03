# 📋 TechTown Payroll Backend — Open Issues

This document tracks open issues for the `techtown-payroll-backend` repository.
Each issue includes its difficulty level, labels, and enough context to get started without needing to dig through the entire codebase first.

> **Want to contribute?** Read [CONTRIBUTING.md](CONTRIBUTING.md), pick an issue, comment on the corresponding GitHub issue to claim it, then open a PR. All skill levels are welcome!

---

## Table of Contents

- [Good First Issues](#good-first-issues)
- [Enhancements](#enhancements)
- [Testing](#testing)
- [Documentation](#documentation)
- [DevOps & Infrastructure](#devops--infrastructure)

---

## Good First Issues

---

### #1 — Return consistent error shapes across all API handlers
**Labels:** `good first issue` · `api` · `dx`
**Difficulty:** ⭐ Beginner

**Description:**
Some handlers return `{ "error": "..." }` while others return plain strings or different field names. Clients can't reliably parse errors.

**Task:**
Define a shared `ApiError { code: String, message: String }` struct in a new `api/error.rs` module. Update all existing handlers to return this type. Implement `IntoResponse` for it.

**Acceptance Criteria:**
- [ ] All error responses share the same JSON shape
- [ ] `code` is a machine-readable constant (e.g. `"EMPLOYEE_NOT_FOUND"`)
- [ ] Existing tests still pass

---

### #2 — Add `GET /api/daos/:id/employees/:employee_id` endpoint
**Labels:** `good first issue` · `api`
**Difficulty:** ⭐ Beginner

**Description:**
There is no single-employee fetch endpoint. Clients must list all employees and filter, which is wasteful.

**Task:**
Add the route in `api/employee.rs`. Query the DB by `(dao_id, employee_id)`. Return 404 if not found.

**Acceptance Criteria:**
- [ ] Endpoint returns a single `Employee` JSON object
- [ ] Returns `404` for unknown IDs
- [ ] Covered by an integration test

---

### #3 — Add request logging middleware
**Labels:** `good first issue` · `observability`
**Difficulty:** ⭐ Beginner

**Description:**
There is no structured request log, making it hard to debug issues in staging.

**Task:**
Add `tower_http::trace::TraceLayer` to the Axum router in `main.rs`. Configure it to log method, path, status, and latency at `INFO` level.

**Acceptance Criteria:**
- [ ] Every HTTP request produces a log line
- [ ] Log line includes method, path, status code, and response time in ms
- [ ] Sensitive headers (e.g. `Authorization`) are not logged

---

### #4 — Add `GET /api/health` detailed response
**Labels:** `good first issue` · `observability`
**Difficulty:** ⭐ Beginner

**Description:**
`/api/health` currently returns a static OK. Operators need to know whether the DB and Redis connections are alive.

**Task:**
Extend `api/health.rs` to ping PostgreSQL and Redis. Return a JSON body like:

```json
{
  "status": "ok",
  "db": "ok",
  "redis": "ok",
  "version": "0.1.0"
}
```

Return `503` if any dependency is unreachable.

**Acceptance Criteria:**
- [ ] Response includes `db`, `redis`, and `version` fields
- [ ] Returns `200` when healthy and `503` when degraded
- [ ] Unit test mocks both healthy and degraded states

---

### #5 — Extract DB query strings into a `queries` module
**Labels:** `good first issue` · `refactor`
**Difficulty:** ⭐ Beginner

**Description:**
SQL strings are currently inlined inside handler functions, making them hard to reuse or review.

**Task:**
Create `db/queries.rs`. Move all SQL string constants there, grouped by resource. Import them in the relevant API modules.

**Acceptance Criteria:**
- [ ] No raw SQL string literals remain in `api/` files
- [ ] All SQL is in named constants with a one-line comment describing the query

---

## Enhancements

---

### #6 — Implement JWT refresh token rotation
**Labels:** `enhancement` · `auth` · `security`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
`/api/auth/refresh` reissues a token but does not invalidate the previous one, allowing replay attacks if a refresh token is leaked.

**Task:**
Store refresh tokens in Redis with a TTL equal to `JWT_EXPIRATION`. On refresh, delete the old token and issue a new one. Return `401` if the presented token is not in Redis.

**Acceptance Criteria:**
- [ ] Old refresh token is invalidated immediately upon refresh
- [ ] Replaying the old token returns `401`
- [ ] TTL is configurable via environment variable

---

### #7 — Add pagination to employee list endpoint
**Labels:** `enhancement` · `api`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
`GET /api/daos/:id/employees` returns all employees at once. Large DAOs will hit performance issues.

**Task:**
Add `?page=1&per_page=20` query parameters. Use `LIMIT`/`OFFSET` in the SQL query. Return a pagination envelope:

```json
{
  "data": [...],
  "page": 1,
  "per_page": 20,
  "total": 150
}
```

**Acceptance Criteria:**
- [ ] Default `per_page` is 20, max is 100
- [ ] `total` reflects the full unfiltered count
- [ ] Tests for first page, last page, and out-of-bounds page

---

### #8 — Add rate limiting per wallet address
**Labels:** `enhancement` · `security`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
Auth endpoints have no rate limiting, making brute-force attacks trivial.

**Task:**
Use Redis to implement a sliding-window rate limiter. Apply it as Axum middleware on `/api/auth/*`. Allow configuring the window and max requests via environment variables.

**Acceptance Criteria:**
- [ ] Requests beyond the limit return `429 Too Many Requests` with a `Retry-After` header
- [ ] Limit is per IP + wallet address pair
- [ ] Configurable via `RATE_LIMIT_WINDOW_SECS` and `RATE_LIMIT_MAX_REQUESTS` env vars

---

### #9 — Replace prototype ZK prover with Arkworks Groth16
**Labels:** `enhancement` · `zk-proofs` · `help wanted`
**Difficulty:** ⭐⭐⭐ Advanced

**Description:**
`services/zk_prover.rs` uses SHA-256 as a placeholder. A real Groth16 prover is needed to generate valid proofs that the on-chain contract can verify.

**Task:**
Integrate `arkworks` (or `bellman`) to generate a Groth16 proof for the payroll circuit. The circuit inputs are `(total_amount, employee_count, merkle_root)`. Return the serialised proof bytes and public inputs.

**References:**
- https://github.com/arkworks-rs/groth16
- `zk_verifier.rs` in `techtown-payroll-contracts` for expected input format

**Acceptance Criteria:**
- [ ] Proof generation produces bytes accepted by the on-chain `verify_payroll_proof`
- [ ] Unit test generates a proof and verifies it locally
- [ ] Old placeholder is removed

---

### #10 — Add soft-delete and audit log for employee changes
**Labels:** `enhancement` · `compliance`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
Employee removal currently performs a hard delete or status update with no record of who made the change or when.

**Task:**
Add an `employee_audit_log` table (migration) with columns `(id, dao_id, employee_id, action, performed_by, performed_at)`. Write a log entry on every employee status change. Expose `GET /api/daos/:id/employees/:employee_id/audit` to return the log.

**Acceptance Criteria:**
- [ ] Migration creates the table
- [ ] All employee mutations write an audit entry
- [ ] Audit endpoint returns entries newest-first
- [ ] Test covers add → freeze → activate → remove sequence

---

### #11 — Support filtering proposals by status
**Labels:** `enhancement` · `api`
**Difficulty:** ⭐ Beginner–Intermediate

**Description:**
`GET /api/daos/:id/proposals` returns all proposals. Most UIs only need pending or approved proposals.

**Task:**
Add an optional `?status=pending|approved|rejected|executed` query parameter. Apply it as a `WHERE` clause.

**Acceptance Criteria:**
- [ ] Without `?status`, all proposals are returned
- [ ] With `?status=pending`, only pending proposals are returned
- [ ] Invalid status values return `400` with a descriptive message

---

## Testing

---

### #12 — Add integration test suite with a test database
**Labels:** `testing`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
There are no integration tests that run against a real database. All current tests (if any) are unit tests.

**Task:**
Set up a test helper that creates an isolated PostgreSQL schema per test run (using `sqlx` test transactions or a dedicated test DB). Write integration tests for at least: register → login → create DAO → add employee → list employees.

**Acceptance Criteria:**
- [ ] Tests run with `cargo test` and require no manual DB setup beyond the standard `.env`
- [ ] Each test runs in its own transaction (rolled back after the test)
- [ ] At least 5 integration tests covering the happy path of each major resource

---

### #13 — Add unit tests for `merkle_tree.rs`
**Labels:** `testing` · `good first issue`
**Difficulty:** ⭐ Beginner

**Description:**
`services/utils/merkle_tree.rs` has no tests. This module is critical for payroll correctness.

**Task:**
Write unit tests covering:
- Merkle root stability (same inputs → same root)
- Single-leaf tree
- Even and odd leaf counts
- Inclusion proof generation and verification

**Acceptance Criteria:**
- [ ] Tests pass with `cargo test`
- [ ] Both even and odd leaf counts are tested
- [ ] A corrupted proof is rejected

---

### #14 — Add property-based tests for commitment hashing
**Labels:** `testing` · `security`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
`zk_prover.rs` generates commitment hashes. Property tests can verify that the function is deterministic and that small input changes produce completely different hashes (avalanche effect).

**Task:**
Using `proptest`, write property tests that assert:
1. Same inputs always produce the same hash
2. Flipping any single byte of the input changes the output

**Acceptance Criteria:**
- [ ] Property tests run as part of `cargo test`
- [ ] Both properties verified over at least 1000 generated cases

---

## Documentation

---

### #15 — Write `CONTRIBUTING.md`
**Labels:** `documentation` · `good first issue`
**Difficulty:** ⭐ Beginner

**Description:**
There is no contribution guide. New contributors do not know how to set up Postgres, Redis, or the Stellar RPC locally.

**Task:**
Create `CONTRIBUTING.md` covering:
- Full local dev setup (Postgres, Redis, Stellar RPC)
- Running migrations
- Running tests
- Environment variable reference
- PR and commit conventions

**Acceptance Criteria:**
- [ ] A developer with Rust knowledge can follow the guide from scratch without asking questions
- [ ] Every env var in `.env.example` is explained

---

### #16 — Generate and host OpenAPI spec
**Labels:** `documentation` · `dx`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
There is no machine-readable API contract. Frontend developers and external integrators have to rely on the README.

**Task:**
Add `utoipa` (or `aide`) to generate an OpenAPI 3.0 spec from the existing Axum routes. Serve the spec at `GET /api/openapi.json` and serve Swagger UI at `GET /api/docs`.

**Acceptance Criteria:**
- [ ] All existing endpoints appear in the spec
- [ ] Request/response schemas are documented
- [ ] Spec is also written to `docs/openapi.json` and committed

---

## DevOps & Infrastructure

---

### #17 — Add `docker-compose.test.yml` for CI integration tests
**Labels:** `ci` · `testing`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
The existing `docker-compose.yml` is for development. CI needs an isolated compose file that spins up Postgres and Redis, runs migrations, runs tests, and exits cleanly.

**Task:**
Create `docker-compose.test.yml` and a `scripts/test.sh` that CI calls. Update the GitHub Actions workflow to use it.

**Acceptance Criteria:**
- [ ] `docker-compose -f docker-compose.test.yml up --abort-on-container-exit` exits `0` on a green test run
- [ ] Containers are removed after the run
- [ ] CI workflow calls the script

---

### #18 — Add `cargo-deny` to CI for dependency auditing
**Labels:** `ci` · `security` · `good first issue`
**Difficulty:** ⭐ Beginner

**Description:**
There is no automated check for known vulnerabilities in Rust dependencies.

**Task:**
Add a `deny.toml` configuration for `cargo-deny` and a CI step that runs `cargo deny check`. Configure it to fail on `deny` level advisories and warn on `unmaintained`.

**Acceptance Criteria:**
- [ ] CI fails if a high-severity advisory is detected
- [ ] `deny.toml` is committed to the repo root
- [ ] Instructions for updating the advisory database are added to `CONTRIBUTING.md`

---

### #19 — Implement graceful shutdown
**Labels:** `enhancement` · `reliability`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
The server in `main.rs` does not handle `SIGTERM`/`SIGINT`. In Kubernetes or Docker this causes in-flight requests to be dropped.

**Task:**
Use `tokio::signal` to listen for `SIGTERM` and `SIGINT`. Call Axum's graceful shutdown. Allow a configurable drain timeout (default 30 s) before forcing exit.

**Acceptance Criteria:**
- [ ] In-flight requests complete before the server exits on `SIGTERM`
- [ ] Drain timeout is configurable via `SHUTDOWN_TIMEOUT_SECS` env var
- [ ] Log message emitted at start of shutdown and on clean exit

---

### #20 — Add structured JSON logging with log levels
**Labels:** `observability` · `enhancement`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
Current logging uses default `tracing` output, which is human-readable but hard to parse in log aggregators (Datadog, CloudWatch, etc.).

**Task:**
Add `tracing-subscriber` with `tracing-subscriber::fmt::json()` format. Make the log format switchable via `LOG_FORMAT=json|pretty` environment variable. Set log level via `RUST_LOG`.

**Acceptance Criteria:**
- [ ] `LOG_FORMAT=json` produces newline-delimited JSON logs
- [ ] `LOG_FORMAT=pretty` (default) produces human-readable output
- [ ] Every log line in JSON mode has at least `timestamp`, `level`, `target`, and `message` fields

---

---

### #21 — Add `DELETE /api/daos/:id/employees/:employee_id` with soft delete
**Labels:** `enhancement` · `api` · `good first issue`
**Difficulty:** ⭐ Beginner

**Description:**
Employee removal currently hard-deletes the database row. This breaks historical payroll records that reference the employee. A soft delete (`status = 'removed'`) preserves referential integrity.

**Task:**
Update the `DELETE` handler in `api/employee.rs` to set `status = 'removed'` and `removed_at = NOW()` instead of deleting the row. Ensure the employee no longer appears in the active employee list.

**Acceptance Criteria:**
- [ ] Row is not deleted; `status` is set to `removed`
- [ ] `GET /api/daos/:id/employees` excludes removed employees by default
- [ ] An optional `?include_removed=true` query param returns all employees
- [ ] Integration test covers remove → list → list-with-flag

---

### #22 — Add request body size limit middleware
**Labels:** `security` · `enhancement`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
The Axum server has no maximum request body size. A malicious client can send arbitrarily large payloads, exhausting memory.

**Task:**
Apply `tower_http::limit::RequestBodyLimitLayer` globally in `main.rs`. Set a default limit of 1 MB. Make it configurable via `MAX_BODY_SIZE_BYTES` environment variable.

**Acceptance Criteria:**
- [ ] Requests exceeding the limit return `413 Payload Too Large`
- [ ] Limit is configurable without recompiling
- [ ] Unit test sends an oversized body and asserts `413`

---

### #23 — Implement idempotency keys for payroll creation
**Labels:** `enhancement` · `api` · `reliability`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
A network timeout during `POST /api/daos/:id/payroll` can cause the client to retry, creating duplicate payroll records for the same period.

**Task:**
Accept an optional `Idempotency-Key` header on the payroll creation endpoint. Store the key in Redis with a 24-hour TTL. If the same key is presented again, return the original response without re-executing the handler.

**Acceptance Criteria:**
- [ ] Identical `Idempotency-Key` on repeat request returns the same `201` response without a new DB row
- [ ] Different keys always create new records
- [ ] TTL is 24 hours; expired keys are treated as new
- [ ] Test covers: first call (creates), second call (idempotent), expired key (creates again)

---

### #24 — Add `GET /api/daos/:id/treasury/transactions` history endpoint
**Labels:** `enhancement` · `api` · `treasury`
**Difficulty:** ⭐⭐ Intermediate

**Description:**
The treasury module tracks deposits and withdrawals in `treasury_transactions` but there is no endpoint to query the history.

**Task:**
Add `GET /api/daos/:id/treasury/transactions` returning a paginated list of `TreasuryTransaction` records ordered by `created_at` descending. Support `?page` and `?per_page` query params.

**Acceptance Criteria:**
- [ ] Endpoint returns all transaction types (deposit, withdrawal)
- [ ] Default `per_page` is 20, max is 100
- [ ] Response includes a `total` count field
- [ ] Integration test with at least two deposits verifies ordering

---

### #25 — Add `SECURITY.md` with responsible disclosure policy
**Labels:** `documentation` · `security` · `good first issue`
**Difficulty:** ⭐ Beginner

**Description:**
The backend processes JWT tokens and on-chain transactions but has no documented process for reporting security vulnerabilities.

**Task:**
Create `SECURITY.md` at the repo root. Include:
- Scope of the security policy
- How to report (private GitHub advisory or email)
- Expected SLA for response
- Known limitations (prototype ZK prover not production-ready)

**Acceptance Criteria:**
- [ ] File exists at repo root with all four sections
- [ ] Contact method is valid and reachable
- [ ] Reviewed by a maintainer

---

*Last updated: 2026-07-03 · Maintainers: TechTown-Private-DAO*
