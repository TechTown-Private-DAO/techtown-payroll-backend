mod api;
mod db;
mod models;
mod services;
mod config;

use axum::{
    routing::{get, post, put, delete},
    Router,
    http::Method,
    Extension,
};
use tower_http::cors::{CorsLayer, Any};
use tracing_subscriber;
use std::sync::Arc;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("techtown_payroll_backend=debug,tower_http=debug")
        .init();

    let config = config::Config::from_env()?;
    let db_pool = db::init_pool(&config.database_url).await?;
    let redis_client = redis::Client::open(config.redis_url.clone())?;

    let stellar_service = services::StellarService::new(
        config.stellar_rpc_url.clone(),
        config.stellar_network_passphrase.clone(),
    );

    let payroll_service = services::PayrollService::new(
        db_pool.clone(),
        stellar_service,
        redis_client.clone(),
    );

    let app = Router::new()
        // health
        .route("/api/health", get(api::health_check))
        // DAO
        .route("/api/daos", post(api::create_dao))
        .route("/api/daos/:id", get(api::get_dao))
        // Employees
        .route("/api/daos/:id/employees", post(api::add_employee))
        .route("/api/daos/:id/employees", get(api::get_employees))
        .route("/api/daos/:id/employees/:employee_id", put(api::update_employee))
        .route("/api/daos/:id/employees/:employee_id", delete(api::remove_employee))
        // Treasury
        .route("/api/daos/:id/treasury/deposit", post(api::deposit_to_treasury))
        .route("/api/daos/:id/treasury/balance", get(api::get_treasury_balance))
        // Payroll
        .route("/api/daos/:id/payroll", post(api::create_payroll))
        .route("/api/daos/:id/payroll", get(api::get_payrolls))
        .route("/api/daos/:id/payroll/:payroll_id/approve", post(api::approve_payroll))
        .route("/api/daos/:id/payroll/:payroll_id/execute", post(api::execute_payroll))
        .route("/api/daos/:id/payroll/:payroll_id/claim", post(api::claim_payroll))
        // Proposals
        .route("/api/daos/:id/proposals", post(api::create_proposal))
        .route("/api/daos/:id/proposals", get(api::get_proposals))
        .route("/api/daos/:id/proposals/:proposal_id/approve", post(api::approve_proposal))
        // Auth
        .route("/api/auth/login", post(api::login))
        .route("/api/auth/register", post(api::register))
        .route("/api/auth/refresh", post(api::refresh_token))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                .allow_origin(Any)
                .allow_headers(Any),
        )
        .layer(Extension(Arc::new(payroll_service)))
        .layer(Extension(Arc::new(config)));

    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;
    tracing::info!("Backend running on http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
