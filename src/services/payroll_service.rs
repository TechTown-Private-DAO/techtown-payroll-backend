use crate::models::*;
use crate::services::StellarService;
use crate::services::utils::merkle_tree::MerkleTree;
use crate::services::utils::zk_prover::ZKProver;
use sqlx::postgres::PgPool;
use redis::Client;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub struct PayrollService {
    db: Arc<PgPool>,
    stellar: Arc<StellarService>,
    redis: Arc<Client>,
    zk_prover: Arc<ZKProver>,
}

impl PayrollService {
    pub fn new(
        db: PgPool,
        stellar: StellarService,
        redis: Client,
    ) -> Self {
        Self {
            db: Arc::new(db),
            stellar: Arc::new(stellar),
            redis: Arc::new(redis),
            zk_prover: Arc::new(ZKProver::new()),
        }
    }

    pub async fn create_dao(
        &self,
        name: String,
        symbol: String,
        admin_address: String,
        multisig_threshold: i32,
    ) -> Result<DAO, Box<dyn std::error::Error>> {
        // Create contract on Stellar
        let contract_address = self.stellar.create_dao_contract(
            &admin_address,
            &name,
            &symbol,
            multisig_threshold,
        ).await?;

        let dao = sqlx::query_as!(
            DAO,
            r#"
            INSERT INTO daos (name, symbol, admin_address, multisig_threshold, contract_address)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, symbol, admin_address, multisig_threshold, total_members, paused, contract_address, created_at, updated_at
            "#,
            name,
            symbol,
            admin_address,
            multisig_threshold,
            contract_address,
        )
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(dao)
    }

    pub async fn add_employee(
        &self,
        dao_id: i64,
        wallet_address: String,
        department: String,
        salary: i128,
        randomness: Vec<u8>,
    ) -> Result<Employee, Box<dyn std::error::Error>> {
        // Generate commitment hash
        let commitment_hash = self.zk_prover.generate_commitment_hash(
            salary,
            &randomness,
            &wallet_address,
        );

        // Store commitment
        let employee = sqlx::query_as!(
            Employee,
            r#"
            INSERT INTO employees (dao_id, wallet_address, department, status, commitment_hash)
            VALUES ($1, $2, $3, 'active', $4)
            RETURNING id, dao_id, wallet_address, department, status, commitment_hash, joined_at, last_payroll_at
            "#,
            dao_id,
            wallet_address,
            department,
            commitment_hash.to_string(),
        )
        .fetch_one(self.db.as_ref())
        .await?;

        Ok(employee)
    }

    pub async fn create_payroll(
        &self,
        dao_id: i64,
        period: DateTime<Utc>,
        employee_ids: Vec<i64>,
    ) -> Result<Payroll, Box<dyn std::error::Error>> {
        // Get employees and their commitments
        let employees = sqlx::query_as!(
            Employee,
            r#"
            SELECT e.id, e.dao_id, e.wallet_address, e.department, e.status, e.commitment_hash, e.joined_at, e.last_payroll_at
            FROM employees e
            WHERE e.dao_id = $1 AND e.id = ANY($2) AND e.status = 'active'
            "#,
            dao_id,
            &employee_ids,
        )
        .fetch_all(self.db.as_ref())
        .await?;

        // Calculate total amount from commitments
        let mut total_amount = 0;
        let mut leaves = Vec::new();
        
        for employee in &employees {
            // In production, this would decrypt the commitment to get the salary
            // For now, we'll assume the salary is stored encrypted
            let salary = 100000; // Example amount
            total_amount += salary;
            leaves.push((employee.id, salary));
        }

        // Generate Merkle tree
        let merkle_tree = MerkleTree::new(leaves);
        let merkle_root = merkle_tree.get_root();

        // Create payroll record
        let payroll = sqlx::query_as!(
            Payroll,
            r#"
            INSERT INTO payrolls (dao_id, period, total_amount, employee_count, status, merkle_root)
            VALUES ($1, $2, $3, $4, 'pending', $5)
            RETURNING id, dao_id, period, total_amount, employee_count, status, merkle_root, created_at, approved_at, executed_at
            "#,
            dao_id,
            period,
            total_amount,
            employees.len() as i32,
            merkle_root,
        )
        .fetch_one(self.db.as_ref())
        .await?;

        // Store commitments
        for employee in &employees {
            sqlx::query!(
                r#"
                INSERT INTO salary_commitments (dao_id, employee_id, commitment_hash, amount, period)
                VALUES ($1, $2, $3, $4, $5)
                "#,
                dao_id,
                employee.id,
                employee.commitment_hash,
                100000, // Example amount
                period,
            )
            .execute(self.db.as_ref())
            .await?;
        }

        // Cache in Redis for fast access
        let mut conn = self.redis.get_async_connection().await?;
        let _: () = redis::cmd("SET")
            .arg(&format!("payroll:{}", payroll.id))
            .arg(serde_json::to_string(&payroll)?)
            .arg("EX")
            .arg(3600)
            .query_async(&mut conn)
            .await?;

        Ok(payroll)
    }

    pub async fn approve_payroll(
        &self,
        payroll_id: i64,
        approver_address: String,
    ) -> Result<Payroll, Box<dyn std::error::Error>> {
        // Get payroll
        let mut payroll: Payroll = sqlx::query_as!(
            Payroll,
            r#"
            SELECT id, dao_id, period, total_amount, employee_count, status, merkle_root, created_at, approved_at, executed_at
            FROM payrolls
            WHERE id = $1
            "#,
            payroll_id,
        )
        .fetch_one(self.db.as_ref())
        .await?;

        if payroll.status != "pending" {
            return Err("Payroll is not in pending state".into());
        }

        // Update payroll
        payroll.status = "approved".to_string();
        payroll.approved_at = Some(Utc::now());

        let updated = sqlx::query_as!(
            Payroll,
            r#"
            UPDATE payrolls
            SET status = $1, approved_at = $2
            WHERE id = $3
            RETURNING id, dao_id, period, total_amount, employee_count, status, merkle_root, created_at, approved_at, executed_at
            "#,
            payroll.status,
            payroll.approved_at,
            payroll_id,
        )
        .fetch_one(self.db.as_ref())
        .await?;

        // Update cache
        let mut conn = self.redis.get_async_connection().await?;
        let _: () = redis::cmd("SET")
            .arg(&format!("payroll:{}", payroll.id))
            .arg(serde_json::to_string(&updated)?)
            .arg("EX")
            .arg(3600)
            .query_async(&mut conn)
            .await?;

        Ok(updated)
    }

    pub async fn execute_payroll(
        &self,
        payroll_id: i64,
        executor_address: String,
    ) -> Result<Payroll, Box<dyn std::error::Error>> {
        // Get payroll
        let payroll: Payroll = sqlx::query_as!(
            Payroll,
            r#"
            SELECT id, dao_id, period, total_amount, employee_count, status, merkle_root, created_at, approved_at, executed_at
            FROM payrolls
            WHERE id = $1
            "#,
            payroll_id,
        )
        .fetch_one(self.db.as_ref())
        .await?;

        if payroll.status != "approved" {
            return Err("Payroll is not approved".into());
        }

        // Get employees and their payments
        let employees = sqlx::query_as!(
            Employee,
            r#"
            SELECT e.id, e.dao_id, e.wallet_address, e.department, e.status, e.commitment_hash, e.joined_at, e.last_payroll_at
            FROM employees e
            JOIN salary_commitments s ON s.employee_id = e.id
            WHERE s.period = $1 AND e.dao_id = $2
            "#,
            payroll.period,
            payroll.dao_id,
        )
        .fetch_all(self.db.as_ref())
        .await?;

        // Generate ZK proof
        let proof = self.zk_prover.generate_payroll_proof(
            &employees,
            payroll.total_amount,
            payroll.merkle_root.clone(),
        )?;

        // Execute on Stellar
        self.stellar.execute_payroll_contract(
            payroll_id,
            payroll.dao_id,
            &proof,
        ).await?;

        // Update payroll
        let mut updated = payroll;
        updated.status = "executed".to_string();
        updated.executed_at = Some(Utc::now());

        let result = sqlx::query_as!(
            Payroll,
            r#"
            UPDATE payrolls
            SET status = $1, executed_at = $2
            WHERE id = $3
            RETURNING id, dao_id, period, total_amount, employee_count, status, merkle_root, created_at, approved_at, executed_at
            "#,
            updated.status,
            updated.executed_at,
            payroll_id,
        )
        .fetch_one(self.db.as_ref())
        .await?;

        // Update employee last payroll
        for employee in &employees {
            sqlx::query!(
                r#"
                UPDATE employees
                SET last_payroll_at = $1
                WHERE id = $2
                "#,
                Utc::now(),
                employee.id,
            )
            .execute(self.db.as_ref())
            .await?;
        }

        // Clear cache
        let mut conn = self.redis.get_async_connection().await?;
        let _: () = redis::cmd("DEL")
            .arg(&format!("payroll:{}", payroll.id))
            .query_async(&mut conn)
            .await?;

        Ok(result)
    }

    pub async fn get_payrolls(
        &self,
        dao_id: i64,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Payroll>, Box<dyn std::error::Error>> {
        let payrolls = sqlx::query_as!(
            Payroll,
            r#"
            SELECT id, dao_id, period, total_amount, employee_count, status, merkle_root, created_at, approved_at, executed_at
            FROM payrolls
            WHERE dao_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            dao_id,
            limit,
            offset,
        )
        .fetch_all(self.db.as_ref())
        .await?;

        Ok(payrolls)
    }

    pub async fn claim_payroll(
        &self,
        payroll_id: i64,
        employee_id: i64,
        employee_address: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Verify employee
        let employee = sqlx::query_as!(
            Employee,
            r#"
            SELECT id, dao_id, wallet_address, department, status, commitment_hash, joined_at, last_payroll_at
            FROM employees
            WHERE id = $1 AND wallet_address = $2
            "#,
            employee_id,
            employee_address,
        )
        .fetch_optional(self.db.as_ref())
        .await?;

        if employee.is_none() {
            return Err("Employee not found".into());
        }

        // Check if payroll is executed
        let payroll = sqlx::query!(
            r#"
            SELECT status
            FROM payrolls
            WHERE id = $1
            "#,
            payroll_id,
        )
        .fetch_one(self.db.as_ref())
        .await?;

        if payroll.status != "executed" {
            return Err("Payroll not executed yet".into());
        }

        // Verify the employee is in this payroll
        let commitment = sqlx::query!(
            r#"
            SELECT s.id
            FROM salary_commitments s
            WHERE s.employee_id = $1 AND s.period = (
                SELECT period FROM payrolls WHERE id = $2
            )
            "#,
            employee_id,
            payroll_id,
        )
        .fetch_optional(self.db.as_ref())
        .await?;

        if commitment.is_none() {
            return Err("Employee not in this payroll".into());
        }

        // Claim on Stellar
        self.stellar.claim_payroll(
            payroll_id,
            employee_id,
            &employee_address,
        ).await?;

        Ok(())
    }

    // ── DAO ──────────────────────────────────────────────────────────────────

    pub async fn get_dao(
        &self,
        dao_id: i64,
    ) -> Result<Option<crate::models::DAO>, Box<dyn std::error::Error>> {
        let dao = sqlx::query_as!(
            crate::models::DAO,
            r#"SELECT id, name, symbol, admin_address, multisig_threshold, total_members, paused, contract_address, created_at, updated_at
               FROM daos WHERE id = $1"#,
            dao_id,
        )
        .fetch_optional(self.db.as_ref())
        .await?;
        Ok(dao)
    }

    // ── Employees ─────────────────────────────────────────────────────────────

    pub async fn get_employees(
        &self,
        dao_id: i64,
    ) -> Result<Vec<crate::models::Employee>, Box<dyn std::error::Error>> {
        let list = sqlx::query_as!(
            crate::models::Employee,
            r#"SELECT id, dao_id, wallet_address, department, status, commitment_hash, joined_at, last_payroll_at
               FROM employees WHERE dao_id = $1 AND status != 'removed' ORDER BY joined_at"#,
            dao_id,
        )
        .fetch_all(self.db.as_ref())
        .await?;
        Ok(list)
    }

    pub async fn freeze_employee(
        &self,
        dao_id: i64,
        employee_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "UPDATE employees SET status = 'frozen' WHERE id = $1 AND dao_id = $2",
            employee_id, dao_id,
        )
        .execute(self.db.as_ref())
        .await?;
        Ok(())
    }

    pub async fn activate_employee(
        &self,
        dao_id: i64,
        employee_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "UPDATE employees SET status = 'active' WHERE id = $1 AND dao_id = $2",
            employee_id, dao_id,
        )
        .execute(self.db.as_ref())
        .await?;
        Ok(())
    }

    pub async fn remove_employee(
        &self,
        dao_id: i64,
        employee_id: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            "UPDATE employees SET status = 'removed' WHERE id = $1 AND dao_id = $2",
            employee_id, dao_id,
        )
        .execute(self.db.as_ref())
        .await?;
        Ok(())
    }

    // ── Treasury ──────────────────────────────────────────────────────────────

    pub async fn deposit_to_treasury(
        &self,
        dao_id: i64,
        token_address: String,
        from_address: String,
        amount: i64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"INSERT INTO treasury_transactions (dao_id, token_address, from_address, amount, tx_type)
               VALUES ($1, $2, $3, $4, 'deposit')"#,
            dao_id, token_address, from_address, amount as i64,
        )
        .execute(self.db.as_ref())
        .await?;
        Ok(())
    }

    pub async fn get_treasury_balance(
        &self,
        dao_id: i64,
    ) -> Result<i64, Box<dyn std::error::Error>> {
        let row = sqlx::query!(
            r#"SELECT COALESCE(
                SUM(CASE WHEN tx_type='deposit' THEN amount ELSE -amount END), 0
               ) AS balance
               FROM treasury_transactions WHERE dao_id = $1"#,
            dao_id,
        )
        .fetch_one(self.db.as_ref())
        .await?;
        Ok(row.balance.unwrap_or(0))
    }

    // ── Proposals ─────────────────────────────────────────────────────────────

    pub async fn create_proposal(
        &self,
        dao_id: i64,
        proposer_address: String,
        target_address: String,
        function: String,
        args: String,
    ) -> Result<crate::models::Proposal, Box<dyn std::error::Error>> {
        let proposal = sqlx::query_as!(
            crate::models::Proposal,
            r#"INSERT INTO proposals (dao_id, proposer_address, target_address, function, args, status, approvals)
               VALUES ($1, $2, $3, $4, $5, 'active', $6)
               RETURNING id, dao_id, proposer_address, target_address, function, args, status, approvals, created_at, executed_at"#,
            dao_id, proposer_address.clone(), target_address, function, args,
            &vec![proposer_address] as &Vec<String>,
        )
        .fetch_one(self.db.as_ref())
        .await?;
        Ok(proposal)
    }

    pub async fn approve_proposal(
        &self,
        proposal_id: i64,
        approver_address: String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"UPDATE proposals SET approvals = array_append(approvals, $1) WHERE id = $2"#,
            approver_address, proposal_id,
        )
        .execute(self.db.as_ref())
        .await?;
        Ok(())
    }

    pub async fn get_proposals(
        &self,
        dao_id: i64,
    ) -> Result<Vec<crate::models::Proposal>, Box<dyn std::error::Error>> {
        let list = sqlx::query_as!(
            crate::models::Proposal,
            r#"SELECT id, dao_id, proposer_address, target_address, function, args, status, approvals, created_at, executed_at
               FROM proposals WHERE dao_id = $1 ORDER BY created_at DESC"#,
            dao_id,
        )
        .fetch_all(self.db.as_ref())
        .await?;
        Ok(list)
    }
}