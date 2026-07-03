use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    Extension,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

use crate::services::PayrollService;
use crate::models::Employee;

#[derive(Deserialize)]
pub struct AddEmployeeRequest {
    pub wallet_address: String,
    pub department: String,
    /// Salary in stroops (1 XLM = 10_000_000 stroops)
    pub salary: i64,
    pub randomness: Vec<u8>,
}

#[derive(Deserialize)]
pub struct UpdateEmployeeRequest {
    pub action: String, // "freeze" | "activate" | "update_wallet"
    pub new_wallet: Option<String>,
}

pub async fn add_employee(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(dao_id): Path<i64>,
    Json(body): Json<AddEmployeeRequest>,
) -> Result<(StatusCode, Json<Employee>), (StatusCode, Json<Value>)> {
    match svc.add_employee(dao_id, body.wallet_address, body.department, body.salary, body.randomness).await {
        Ok(emp) => Ok((StatusCode::CREATED, Json(emp))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_employees(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(dao_id): Path<i64>,
) -> Result<Json<Vec<Employee>>, (StatusCode, Json<Value>)> {
    match svc.get_employees(dao_id).await {
        Ok(list) => Ok(Json(list)),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn update_employee(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path((dao_id, employee_id)): Path<(i64, i64)>,
    Json(body): Json<UpdateEmployeeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let result = match body.action.as_str() {
        "freeze" => svc.freeze_employee(dao_id, employee_id).await,
        "activate" => svc.activate_employee(dao_id, employee_id).await,
        _ => return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Unknown action" })))),
    };

    match result {
        Ok(_) => Ok(Json(json!({ "success": true }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn remove_employee(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path((dao_id, employee_id)): Path<(i64, i64)>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    match svc.remove_employee(dao_id, employee_id).await {
        Ok(_) => Ok(Json(json!({ "success": true }))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}
