use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    Extension,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use chrono::{DateTime, Utc};

use crate::services::PayrollService;
use crate::models::{Payroll, Proposal};

// ── Payroll ──────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreatePayrollRequest {
    pub period: DateTime<Utc>,
    pub employee_ids: Vec<i64>,
}

#[derive(Deserialize)]
pub struct ApprovePayrollRequest {
    pub approver_address: String,
}

#[derive(Deserialize)]
pub struct ExecutePayrollRequest {
    pub executor_address: String,
}

#[derive(Deserialize)]
pub struct ClaimPayrollRequest {
    pub employee_id: i64,
    pub employee_address: String,
}

#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn create_payroll(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(dao_id): Path<i64>,
    Json(body): Json<CreatePayrollRequest>,
) -> Result<(StatusCode, Json<Payroll>), (StatusCode, Json<Value>)> {
    match svc.create_payroll(dao_id, body.period, body.employee_ids).await {
        Ok(p) => Ok((StatusCode::CREATED, Json(p))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn approve_payroll(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path((_dao_id, payroll_id)): Path<(i64, i64)>,
    Json(body): Json<ApprovePayrollRequest>,
) -> Result<Json<Payroll>, (StatusCode, Json<Value>)> {
    match svc.approve_payroll(payroll_id, body.approver_address).await {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn execute_payroll(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path((_dao_id, payroll_id)): Path<(i64, i64)>,
    Json(body): Json<ExecutePayrollRequest>,
) -> Result<Json<Payroll>, (StatusCode, Json<Value>)> {
    match svc.execute_payroll(payroll_id, body.executor_address).await {
        Ok(p) => Ok(Json(p)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn claim_payroll(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path((_dao_id, payroll_id)): Path<(i64, i64)>,
    Json(body): Json<ClaimPayrollRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match svc.claim_payroll(payroll_id, body.employee_id, body.employee_address).await {
        Ok(_) => Ok(Json(json!({ "success": true }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_payrolls(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(dao_id): Path<i64>,
) -> Result<Json<Vec<Payroll>>, (StatusCode, Json<Value>)> {
    match svc.get_payrolls(dao_id, 50, 0).await {
        Ok(list) => Ok(Json(list)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

// ── Treasury ─────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct DepositRequest {
    pub token_address: String,
    pub from_address: String,
    pub amount: i64,
}

pub async fn deposit_to_treasury(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(dao_id): Path<i64>,
    Json(body): Json<DepositRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match svc.deposit_to_treasury(dao_id, body.token_address, body.from_address, body.amount).await {
        Ok(_) => Ok(Json(json!({ "success": true }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_treasury_balance(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(dao_id): Path<i64>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match svc.get_treasury_balance(dao_id).await {
        Ok(balance) => Ok(Json(json!({ "balance": balance }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

// ── Proposals ────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateProposalRequest {
    pub proposer_address: String,
    pub target_address: String,
    pub function: String,
    pub args: String,
}

#[derive(Deserialize)]
pub struct ApproveProposalRequest {
    pub approver_address: String,
}

pub async fn create_proposal(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(dao_id): Path<i64>,
    Json(body): Json<CreateProposalRequest>,
) -> Result<(StatusCode, Json<Proposal>), (StatusCode, Json<Value>)> {
    match svc.create_proposal(dao_id, body.proposer_address, body.target_address, body.function, body.args).await {
        Ok(p) => Ok((StatusCode::CREATED, Json(p))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn approve_proposal(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path((_dao_id, proposal_id)): Path<(i64, i64)>,
    Json(body): Json<ApproveProposalRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match svc.approve_proposal(proposal_id, body.approver_address).await {
        Ok(_) => Ok(Json(json!({ "success": true }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_proposals(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(dao_id): Path<i64>,
) -> Result<Json<Vec<Proposal>>, (StatusCode, Json<Value>)> {
    match svc.get_proposals(dao_id).await {
        Ok(list) => Ok(Json(list)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}
