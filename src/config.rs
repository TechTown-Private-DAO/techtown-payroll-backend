use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub stellar_rpc_url: String,
    pub stellar_network_passphrase: String,
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub cors_origin: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        Ok(Config {
            database_url: env::var("DATABASE_URL")?,
            redis_url: env::var("REDIS_URL")?,
            stellar_rpc_url: env::var("STELLAR_RPC_URL")?,
            stellar_network_passphrase: env::var("STELLAR_NETWORK_PASSPHRASE")?,
            jwt_secret: env::var("JWT_SECRET")?,
            jwt_expiration: env::var("JWT_EXPIRATION")?.parse()?,
            cors_origin: env::var("CORS_ORIGIN").unwrap_or_else(|_| "*".to_string()),
            port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()).parse()?,
        })
    }
}