use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub wallet_address: String,
    pub username: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAO {
    pub id: i64,
    pub name: String,
    pub symbol: String,
    pub admin_address: String,
    pub multisig_threshold: i32,
    pub total_members: i32,
    pub paused: bool,
    pub contract_address: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: i64,
    pub dao_id: i64,
    pub wallet_address: String,
    pub department: String,
    pub status: String,
    pub commitment_hash: String,
    pub joined_at: DateTime<Utc>,
    pub last_payroll_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payroll {
    pub id: i64,
    pub dao_id: i64,
    pub period: DateTime<Utc>,
    pub total_amount: i128,
    pub employee_count: i32,
    pub status: String,
    pub merkle_root: String,
    pub created_at: DateTime<Utc>,
    pub approved_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SalaryCommitment {
    pub id: i64,
    pub dao_id: i64,
    pub employee_id: i64,
    pub commitment_hash: String,
    pub amount: i128,
    pub period: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: i64,
    pub dao_id: i64,
    pub proposer_address: String,
    pub target_address: String,
    pub function: String,
    pub args: String,
    pub status: String,
    pub approvals: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i64,
    pub dao_id: i64,
    pub tx_hash: String,
    pub function: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}
