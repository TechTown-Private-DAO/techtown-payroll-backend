<div align="center">

# 🔐 TechTown Payroll — Backend

**A privacy-first payroll API for DAOs, built on Stellar/Soroban**

![Rust](https://img.shields.io/badge/Rust-2021-orange?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.7-blue)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-14+-336791?logo=postgresql&logoColor=white)
![Redis](https://img.shields.io/badge/Redis-6+-DC382D?logo=redis&logoColor=white)
![Stellar](https://img.shields.io/badge/Stellar-Soroban-7B61FF?logo=stellar&logoColor=white)
![License](https://img.shields.io/badge/license-MIT-green)

</div>

---

## What is this?

TechTown Payroll Backend is a REST API server that powers confidential DAO payroll on the Stellar blockchain. Salaries are never stored in plaintext — instead, they are locked behind **Zero-Knowledge commitment hashes** and **Merkle trees**, allowing payroll to be verified on-chain without revealing individual amounts.

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| 🏛️ **DAO Management** | Create and manage on-chain DAOs with configurable multi-sig thresholds |
| 👥 **Employee Lifecycle** | Add, freeze, activate, and remove employees with ZK salary commitments |
| 💸 **Payroll Lifecycle** | Full flow: Create → Approve → Execute → Claim with on-chain ZK proof |
| 🏦 **Treasury** | Token deposits and live balance tracking across all transactions |
| 🗳️ **Proposals** | Multi-sig governance proposals with per-address approval tracking |
| 🔑 **Auth** | Wallet-based JWT authentication (register, login, refresh) |
| ⚡ **Caching** | Redis-backed payroll cache with 1-hour TTL and auto-invalidation |

---

## 🏗️ Architecture

```
techtown-payroll-backend/
├── src/
│   ├── main.rs                    # Server bootstrap and router setup
│   ├── config.rs                  # Environment-based configuration
│   ├── models.rs                  # Shared data types (DAO, Employee, Payroll…)
│   ├── api/
│   │   ├── auth.rs                # Login, register, token refresh
│   │   ├── dao.rs                 # Create and fetch DAOs
│   │   ├── employee.rs            # Employee CRUD
│   │   ├── payroll.rs             # Payroll, treasury, proposals
│   │   └── health.rs              # Health check
│   ├── db/
│   │   └── mod.rs                 # PostgreSQL connection pool
│   └── services/
│       ├── payroll_service.rs     # Core business logic
│       └── utils/
│           ├── merkle_tree.rs     # Merkle tree construction
│           └── zk_prover.rs      # ZK commitment and proof generation
└── migrations/
    └── 0001_initial.sql           # Full database schema
```

---

## 🔌 API Reference

<details>
<summary><b>🔒 Auth</b></summary>

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/auth/register` | Register a new wallet user |
| `POST` | `/api/auth/login` | Authenticate and receive a JWT |
| `POST` | `/api/auth/refresh` | Refresh an existing JWT |

</details>

<details>
<summary><b>🏛️ DAOs</b></summary>

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/daos` | Create a new DAO |
| `GET` | `/api/daos/:id` | Get DAO by ID |

</details>

<details>
<summary><b>👥 Employees</b></summary>

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/daos/:id/employees` | Add an employee |
| `GET` | `/api/daos/:id/employees` | List all active employees |
| `PUT` | `/api/daos/:id/employees/:employee_id` | Freeze or activate an employee |
| `DELETE` | `/api/daos/:id/employees/:employee_id` | Remove an employee |

</details>

<details>
<summary><b>💸 Payroll</b></summary>

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/daos/:id/payroll` | Create a payroll run |
| `GET` | `/api/daos/:id/payroll` | List payrolls (paginated) |
| `POST` | `/api/daos/:id/payroll/:payroll_id/approve` | Approve a payroll |
| `POST` | `/api/daos/:id/payroll/:payroll_id/execute` | Execute an approved payroll on-chain |
| `POST` | `/api/daos/:id/payroll/:payroll_id/claim` | Claim payment as an employee |

</details>

<details>
<summary><b>🏦 Treasury</b></summary>

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/daos/:id/treasury/deposit` | Deposit tokens to treasury |
| `GET` | `/api/daos/:id/treasury/balance` | Get current treasury balance |

</details>

<details>
<summary><b>🗳️ Proposals</b></summary>

| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/daos/:id/proposals` | Create a governance proposal |
| `GET` | `/api/daos/:id/proposals` | List proposals |
| `POST` | `/api/daos/:id/proposals/:proposal_id/approve` | Approve a proposal |

</details>

<details>
<summary><b>❤️ Health</b></summary>

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/health` | Server health check |

</details>

---

## 🗄️ Database Schema

| Table | Purpose |
|-------|---------|
| `users` | Registered wallet users |
| `daos` | DAO registry with on-chain contract address |
| `employees` | DAO members with ZK commitment hashes |
| `payrolls` | Payroll runs with Merkle root and lifecycle status |
| `salary_commitments` | Per-employee salary commitments per payroll period |
| `proposals` | Governance proposals with multi-sig approvals |
| `treasury_transactions` | Deposit/withdrawal records for balance tracking |

---

## 🚀 Getting Started

### Prerequisites

- **Rust** 1.75+
- **PostgreSQL** 14+
- **Redis** 6+
- A **Stellar RPC** endpoint (Soroban-compatible)

### 1. Configure Environment

Create a `.env` file in the project root:

```env
DATABASE_URL=postgres://user:password@localhost:5432/techtown_payroll
REDIS_URL=redis://localhost:6379
STELLAR_RPC_URL=https://soroban-testnet.stellar.org
STELLAR_NETWORK_PASSPHRASE=Test SDF Network ; September 2015
JWT_SECRET=your-super-secret-key
JWT_EXPIRATION=3600
CORS_ORIGIN=http://localhost:3001
PORT=3000
```

### 2. Run Migrations

```bash
cargo install sqlx-cli
sqlx migrate run
```

### 3. Start the Server

```bash
cargo run
```

> Server starts at **http://localhost:3000**

---

## 🐳 Docker

**Standalone:**
```bash
docker build -t techtown-payroll-backend .
docker run -p 3000:3000 --env-file .env techtown-payroll-backend
```

**With Docker Compose** (from the root `xiaxia/` directory):
```bash
docker compose up
```

---

## 🛠️ Development Commands

```bash
cargo watch -x run   # Auto-reload on file changes
cargo test           # Run tests
cargo clippy         # Lint
cargo fmt            # Format code
```

---

## 🔐 Privacy Model

Salary amounts are **never stored in plaintext**. Here's how it works:

```
Employee Added
      │
      ▼
SHA-256(salary ∥ randomness ∥ wallet_address)
      │
      ▼
  commitment_hash  ──────────────► stored in DB + on-chain
      │
      ▼
Payroll Execution
      │
      ▼
MerkleTree(all commitments)
      │
      ▼
  merkle_root  ────────────────── anchored to payroll record on Stellar
```

> ⚠️ **Note:** The current ZK prover is a prototype using SHA-256. For production, replace it with a full **Groth16** or **PLONK** proof system (e.g., [`arkworks`](https://github.com/arkworks-rs) or [`bellman`](https://github.com/zkcrypto/bellman)).

---

## 📄 License

[MIT](./LICENSE)
