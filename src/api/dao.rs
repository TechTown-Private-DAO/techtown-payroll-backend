use axum::{
    extract::Path,
    http::StatusCode,
    Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::services::PayrollService;
use crate::models::DAO;

#[derive(Deserialize)]
pub struct CreateDAORequest {
    pub name: String,
    pub symbol: String,
    pub admin_address: String,
    pub multisig_threshold: i32,
}

pub async fn create_dao(
    Extension(svc): Extension<Arc<PayrollService>>,
    Json(body): Json<CreateDAORequest>,
) -> Result<(StatusCode, Json<DAO>), (StatusCode, Json<Value>)> {
    match svc.create_dao(
        body.name,
        body.symbol,
        body.admin_address,
        body.multisig_threshold,
    ).await {
        Ok(dao) => Ok((StatusCode::CREATED, Json(dao))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}

pub async fn get_dao(
    Extension(svc): Extension<Arc<PayrollService>>,
    Path(id): Path<i64>,
) -> Result<Json<DAO>, (StatusCode, Json<Value>)> {
    match svc.get_dao(id).await {
        Ok(Some(dao)) => Ok(Json(dao)),
        Ok(None) => Err((StatusCode::NOT_FOUND, Json(json!({ "error": "DAO not found" })))),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() })))),
    }
}
