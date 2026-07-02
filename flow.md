# techtown-payroll-backend — Code Flow

This repo is the **Rust/Axum REST API** that sits between the frontend and the Stellar blockchain. It handles auth, mirrors on-chain state into PostgreSQL, generates ZK proofs and Merkle trees off-chain, and calls the Soroban smart contract via the Stellar RPC.

---

## Module Map

```
src/
├── main.rs                     ← App entry point: builds Router, wires services, starts server
├── config.rs                   ← Reads env vars (DATABASE_URL, REDIS_URL, STELLAR_RPC_URL, …)
├── models.rs                   ← Rust structs mirroring DB tables (User, DAO, Employee, Payroll, …)
├── api/
│   ├── mod.rs                  ← Re-exports all handler functions
│   ├── health.rs               ← GET /api/health
│   ├── auth.rs                 ← POST /api/auth/login|register|refresh
│   ├── dao.rs                  ← POST /api/daos, GET /api/daos/:id
│   ├── employee.rs             ← CRUD under /api/daos/:id/employees
│   └── payroll.rs              ← Payroll, treasury, and proposal handlers
├── db/
│   └── mod.rs                  ← Creates sqlx PgPool, runs migrations on startup
├── services/
│   ├── mod.rs                  ← Re-exports PayrollService, StellarService
│   ├── payroll_service.rs      ← Core business logic: ZK proof, Merkle tree, DB writes
│   ├── stellar_service.rs      ← HTTP calls to Stellar RPC (deploy contract, execute payroll, …)
│   └── utils/
│       ├── merkle_tree.rs      ← Builds Merkle tree from (employee_id, amount) leaves
│       └── zk_prover.rs        ← Off-chain ZK proof generation
migrations/
└── 0001_initial.sql            ← Creates all tables: users, daos, employees, payrolls, …
```

---

## Request Lifecycle

```
HTTP Request
     │
     ▼
main.rs (Axum Router)
     │  layer: CorsLayer
     │  layer: Extension<PayrollService>
     │  layer: Extension<Config>
     ▼
api/<handler>.rs
     │  deserialise JSON body
     │  extract Extension<PayrollService>
     ▼
services/payroll_service.rs
     ├─ DB read/write via sqlx (PgPool)
     ├─ ZKProver::generate_proof()    (off-chain)
     ├─ MerkleTree::build()           (off-chain)
     └─ StellarService::*()           (→ Stellar RPC → on-chain contract)
     ▼
JSON response back to client
```

---

## API Routes & What They Do

### Auth
| Method | Path | Handler | What happens |
|---|---|---|---|
| POST | `/api/auth/register` | `register` | Creates user row, issues JWT |
| POST | `/api/auth/login` | `login` | Verifies Stellar wallet signature, issues JWT |
| POST | `/api/auth/refresh` | `refresh_token` | Rotates JWT |

### DAO
| Method | Path | Handler | What happens |
|---|---|---|---|
| POST | `/api/daos` | `create_dao` | Calls `StellarService::create_dao_contract` → inserts into `daos` table |
| GET | `/api/daos/:id` | `get_dao` | Reads from `daos` table |

### Employees
| Method | Path | Handler | What happens |
|---|---|---|---|
| POST | `/api/daos/:id/employees` | `add_employee` | Computes `commitment_hash = H(salary‖randomness‖id)` → inserts into `employees` + `salary_commitments` |
| GET | `/api/daos/:id/employees` | `get_employees` | Reads from `employees` table |
| PUT | `/api/daos/:id/employees/:eid` | `update_employee` | freeze / activate actions |
| DELETE | `/api/daos/:id/employees/:eid` | `remove_employee` | Soft-remove |

### Payroll
| Method | Path | Handler | What happens |
|---|---|---|---|
| POST | `/api/daos/:id/payroll` | `create_payroll` | Fetches salary commitments → builds Merkle tree → generates ZK proof → inserts `Payroll{pending}` |
| GET | `/api/daos/:id/payroll` | `get_payrolls` | Lists payrolls from DB |
| POST | `…/payroll/:pid/approve` | `approve_payroll` | Records approval; when threshold met → `Payroll{approved}` |
| POST | `…/payroll/:pid/execute` | `execute_payroll` | Calls `StellarService::execute_payroll_contract` → updates `Payroll{executed}` |
| POST | `…/payroll/:pid/claim` | `claim_payroll` | Calls `StellarService::claim_payroll` → employee receives tokens |

### Treasury
| Method | Path | Handler | What happens |
|---|---|---|---|
| POST | `/api/daos/:id/treasury/deposit` | `deposit_to_treasury` | Records in `treasury_transactions`, calls Stellar transfer |
| GET | `/api/daos/:id/treasury/balance` | `get_treasury_balance` | Queries Stellar RPC or cached value |

### Proposals (Multisig)
| Method | Path | Handler | What happens |
|---|---|---|---|
| POST | `/api/daos/:id/proposals` | `create_proposal` | Inserts `MultisigProposal{active}` |
| GET | `/api/daos/:id/proposals` | `get_proposals` | Lists proposals |
| POST | `…/proposals/:pid/approve` | `approve_proposal` | Appends approver; executes on-chain when threshold met |

---

## Database Schema (PostgreSQL)

```
users              id, wallet_address, username, email
daos               id, name, symbol, admin_address, multisig_threshold, contract_address
employees          id, dao_id → daos, wallet_address, department, status, commitment_hash
salary_commitments id, dao_id, employee_id → employees, commitment_hash, amount, period
payrolls           id, dao_id → daos, period, total_amount, merkle_root, status
proposals          id, dao_id → daos, proposer_address, function, args, approvals[], status
treasury_transactions id, dao_id → daos, token_address, amount, tx_type
```

---

## Service Wiring

```
main.rs
  ├─ Config::from_env()               → config.rs
  ├─ db::init_pool(DATABASE_URL)      → db/mod.rs  (PgPool + auto-migrate)
  ├─ redis::Client::open(REDIS_URL)   → caching layer
  ├─ StellarService::new(RPC_URL, …)  → services/stellar_service.rs
  └─ PayrollService::new(db, stellar, redis)
          ├─ holds Arc<PgPool>
          ├─ holds Arc<StellarService>
          ├─ holds Arc<RedisClient>
          └─ holds Arc<ZKProver>
```

---

## How this repo connects to the rest of the system

```
techtown-payroll-contracts  (Soroban WASM on Stellar)
        ▲
        │  RPC calls (deploy, execute_payroll, claim, transfer)
        │
techtown-payroll-backend   ◄──── this repo
        │
        │  REST API  (JSON over HTTP, JWT auth)
        │
        ▼
techtown-payroll-web       (Next.js frontend)
```

See `../flow.md` (root) for the full end-to-end picture.
