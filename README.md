# TechTown Payroll Backend

The REST API server for **TechTown Private DAO** — a confidential payroll system built on Stellar/Soroban. It handles DAO management, employee onboarding, payroll lifecycle, treasury operations, governance proposals, and authentication.

## Overview

- **Language:** Rust (2021 edition)
- **Framework:** Axum 0.7
- **Database:** PostgreSQL (via SQLx)
- **Cache:** Redis
- **Blockchain:** Stellar / Soroban smart contracts
- **Privacy:** Zero-Knowledge commitments using SHA-256 + Merkle trees

## Architecture

```
src/
├── main.rs              # Server bootstrap, router setup
├── config.rs            # Config loaded from environment variables
├── models.rs            # Shared data models (DAO, Employee, Payroll, Proposal, etc.)
├── api/
│   ├── auth.rs          # Login, register, token refresh
│   ├── dao.rs           # Create and fetch DAOs
│   ├── employee.rs      # Add, update, remove employees
│   ├── payroll.rs       # Payroll CRUD, treasury, proposals
│   └── health.rs        # Health check endpoint
├── db/
│   └── mod.rs           # Database connection pool setup
└── services/
    ├── mod.rs           # Service exports (PayrollService, StellarService)
    ├── payroll_service.rs # Core business logic
    └── utils/
        ├── merkle_tree.rs # Merkle tree for payroll proofs
        └── zk_prover.rs   # ZK commitment and proof generation
migrations/
└── 0001_initial.sql     # Initial database schema
```

## Features

- **DAO Management** — Create and manage on-chain DAOs with multi-sig thresholds
- **Employee Lifecycle** — Add, freeze, activate, and remove employees with privacy-preserving salary commitments
- **Payroll Lifecycle** — Create → Approve → Execute → Claim with ZK proof verification and Merkle root anchoring
- **Treasury** — Track deposits and compute live balance across all treasury transactions
- **Proposals** — Multi-sig governance proposals with per-address approval tracking
- **Auth** — Wallet-based authentication (JWT) with register, login, and token refresh
- **Caching** — Redis caching for payroll reads with 1-hour TTL and automatic invalidation

## API Reference

### Health
| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/health` | Server health check |

### Auth
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/auth/register` | Register a new wallet user |
| POST | `/api/auth/login` | Authenticate and receive JWT |
| POST | `/api/auth/refresh` | Refresh an existing JWT |

### DAOs
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/daos` | Create a new DAO |
| GET | `/api/daos/:id` | Get DAO by ID |

### Employees
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/daos/:id/employees` | Add an employee |
| GET | `/api/daos/:id/employees` | List all active employees |
| PUT | `/api/daos/:id/employees/:employee_id` | Freeze or activate an employee |
| DELETE | `/api/daos/:id/employees/:employee_id` | Remove an employee |

### Payroll
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/daos/:id/payroll` | Create a payroll run |
| GET | `/api/daos/:id/payroll` | List payrolls (paginated) |
| POST | `/api/daos/:id/payroll/:payroll_id/approve` | Approve a payroll |
| POST | `/api/daos/:id/payroll/:payroll_id/execute` | Execute an approved payroll |
| POST | `/api/daos/:id/payroll/:payroll_id/claim` | Claim payroll as an employee |

### Treasury
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/daos/:id/treasury/deposit` | Deposit tokens to treasury |
| GET | `/api/daos/:id/treasury/balance` | Get current treasury balance |

### Proposals
| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/daos/:id/proposals` | Create a governance proposal |
| GET | `/api/daos/:id/proposals` | List proposals |
| POST | `/api/daos/:id/proposals/:proposal_id/approve` | Approve a proposal |

## Database Schema

| Table | Description |
|-------|-------------|
| `users` | Registered wallet users |
| `daos` | DAO registry with on-chain contract address |
| `employees` | DAO members with ZK commitment hashes |
| `payrolls` | Payroll runs with Merkle root and lifecycle status |
| `salary_commitments` | Per-employee salary commitments per payroll period |
| `proposals` | Governance proposals with multi-sig approvals array |
| `treasury_transactions` | Deposits and withdrawals for treasury balance tracking |

## Getting Started

### Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- Redis 6+
- A Stellar RPC endpoint (Soroban-compatible)

### Environment Variables

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

### Running Locally

```bash
# Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Run database migrations
cargo install sqlx-cli
sqlx migrate run

# Start the server
cargo run
```

The server starts on `http://localhost:3000` by default.

### Using Docker

```bash
docker build -t techtown-payroll-backend .
docker run -p 3000:3000 --env-file .env techtown-payroll-backend
```

### With Docker Compose

From the root `xiaxia/` directory:

```bash
docker compose up
```

## Development

```bash
# Run with auto-reload
cargo watch -x run

# Run tests
cargo test

# Check for linting issues
cargo clippy

# Format code
cargo fmt
```

## Privacy Model

Salary amounts are never stored in plaintext. When an employee is added:

1. A **commitment hash** is generated: `SHA-256(salary || randomness || wallet_address)`
2. Only the hash is stored on-chain and in the database
3. At payroll execution, a **Merkle tree** is built from all salary commitments
4. The **Merkle root** is anchored to the payroll record and verified on Stellar

> Note: The current ZK prover is a prototype using SHA-256 hashing. Production deployment should replace it with a full Groth16 or PLONK proof system (e.g., via `arkworks` or `bellman`).

## License

MIT
