mod api;
mod db;
mod models;
mod services;
mod utils;
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
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("techtown_payroll_backend=debug")
        .init();

    // Load configuration
    let config = config::Config::from_env()?;

    // Initialize database
    let db_pool = db::init_pool(&config.database_url).await?;
    
    // Initialize Redis
    let redis_client = redis::Client::open(config.redis_url.clone())?;

    // Initialize services
    let stellar_service = services::StellarService::new(
        config.stellar_rpc_url,
        config.stellar_network_passphrase,
    );
    
    let payroll_service = services::PayrollService::new(
        db_pool.clone(),
        stellar_service,
        redis_client.clone(),
    );

    // Build application routes
    let app = Router::new()
        .route("/api/health", get(api::health_check))
        .route("/api/daos", post(api::create_dao))
        .route("/api/daos/:id", get(api::get_dao))
        .route("/api/daos/:id/employees", post(api::add_employee))
        .route("/api/daos/:id/employees", get(api::get_employees))
        .route("/api/daos/:id/employees/:employee_id", put(api::update_employee))
        .route("/api/daos/:id/employees/:employee_id", delete(api::remove_employee))
        .route("/api/daos/:id/treasury/deposit", post(api::deposit_to_treasury))
        .route("/api/daos/:id/treasury/balance", get(api::get_treasury_balance))
        .route("/api/daos/:id/payroll", post(api::create_payroll))
        .route("/api/daos/:id/payroll/:payroll_id/approve", post(api::approve_payroll))
        .route("/api/daos/:id/payroll/:payroll_id/execute", post(api::execute_payroll))
        .route("/api/daos/:id/payroll/:payroll_id/claim", post(api::claim_payroll))
        .route("/api/daos/:id/payroll", get(api::get_payrolls))
        .route("/api/daos/:id/proposals", post(api::create_proposal))
        .route("/api/daos/:id/proposals/:proposal_id/approve", post(api::approve_proposal))
        .route("/api/daos/:id/proposals", get(api::get_proposals))
        .route("/api/auth/login", post(api::login))
        .route("/api/auth/register", post(api::register))
        .route("/api/auth/refresh", post(api::refresh_token))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
                .allow_origin(Any)
                .allow_headers(Any)
        )
        .layer(Extension(Arc::new(payroll_service)))
        .layer(Extension(Arc::new(config)));

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server running on http://0.0.0.0:3000");
    
    axum::serve(listener, app).await?;

    Ok(())
}

