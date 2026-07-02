-- Initial schema for TechTown Private DAO

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    wallet_address VARCHAR(56) NOT NULL UNIQUE,
    username VARCHAR(100) NOT NULL,
    email VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS daos (
    id BIGSERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    symbol VARCHAR(20) NOT NULL,
    admin_address VARCHAR(56) NOT NULL,
    multisig_threshold INT NOT NULL DEFAULT 2,
    total_members INT NOT NULL DEFAULT 1,
    paused BOOLEAN NOT NULL DEFAULT FALSE,
    contract_address VARCHAR(56) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS employees (
    id BIGSERIAL PRIMARY KEY,
    dao_id BIGINT NOT NULL REFERENCES daos(id) ON DELETE CASCADE,
    wallet_address VARCHAR(56) NOT NULL,
    department VARCHAR(100) NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active','frozen','removed')),
    commitment_hash VARCHAR(64) NOT NULL,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_payroll_at TIMESTAMPTZ,
    UNIQUE (dao_id, wallet_address)
);

CREATE TABLE IF NOT EXISTS payrolls (
    id BIGSERIAL PRIMARY KEY,
    dao_id BIGINT NOT NULL REFERENCES daos(id) ON DELETE CASCADE,
    period TIMESTAMPTZ NOT NULL,
    total_amount BIGINT NOT NULL DEFAULT 0,
    employee_count INT NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending','approved','executed','cancelled')),
    merkle_root VARCHAR(64) NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMPTZ,
    executed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS salary_commitments (
    id BIGSERIAL PRIMARY KEY,
    dao_id BIGINT NOT NULL REFERENCES daos(id) ON DELETE CASCADE,
    employee_id BIGINT NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    commitment_hash VARCHAR(64) NOT NULL,
    amount BIGINT NOT NULL DEFAULT 0,
    period TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS proposals (
    id BIGSERIAL PRIMARY KEY,
    dao_id BIGINT NOT NULL REFERENCES daos(id) ON DELETE CASCADE,
    proposer_address VARCHAR(56) NOT NULL,
    target_address VARCHAR(56) NOT NULL DEFAULT '',
    function VARCHAR(100) NOT NULL DEFAULT '',
    args TEXT NOT NULL DEFAULT '',
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active','approved','executed','rejected')),
    approvals TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    executed_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS treasury_transactions (
    id BIGSERIAL PRIMARY KEY,
    dao_id BIGINT NOT NULL REFERENCES daos(id) ON DELETE CASCADE,
    token_address VARCHAR(56) NOT NULL,
    from_address VARCHAR(56) NOT NULL DEFAULT '',
    amount BIGINT NOT NULL,
    tx_type VARCHAR(20) NOT NULL CHECK (tx_type IN ('deposit','withdraw')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_employees_dao ON employees(dao_id);
CREATE INDEX IF NOT EXISTS idx_payrolls_dao ON payrolls(dao_id);
CREATE INDEX IF NOT EXISTS idx_salary_commitments_employee ON salary_commitments(employee_id);
CREATE INDEX IF NOT EXISTS idx_proposals_dao ON proposals(dao_id);
CREATE INDEX IF NOT EXISTS idx_treasury_dao ON treasury_transactions(dao_id);
