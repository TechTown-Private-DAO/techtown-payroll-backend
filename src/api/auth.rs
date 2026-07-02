use axum::{http::StatusCode, Json, Extension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use chrono::{Utc, Duration};
use uuid::Uuid;

use crate::config::Config;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,   // wallet address
    pub exp: usize,
    pub iat: usize,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub wallet_address: String,
    /// Signed message proving ownership of the wallet
    pub signature: String,
    pub message: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub wallet_address: String,
    pub username: String,
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub wallet_address: String,
    pub expires_at: i64,
}

fn make_token(wallet: &str, secret: &str, expiry_secs: u64) -> Result<(String, i64), jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let expires_at = now + Duration::seconds(expiry_secs as i64);
    let claims = Claims {
        sub: wallet.to_string(),
        iat: now.timestamp() as usize,
        exp: expires_at.timestamp() as usize,
    };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))?;
    Ok((token, expires_at.timestamp()))
}

pub async fn login(
    Extension(config): Extension<Arc<Config>>,
    Json(body): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    // In production: verify `body.signature` against `body.message` for the wallet.
    // Here we accept any well-formed Stellar public key (56 chars starting with G).
    if body.wallet_address.len() != 56 || !body.wallet_address.starts_with('G') {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid wallet address" })),
        ));
    }

    let (token, expires_at) = make_token(&body.wallet_address, &config.jwt_secret, config.jwt_expiration)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(AuthResponse {
        token,
        wallet_address: body.wallet_address,
        expires_at,
    }))
}

pub async fn register(
    Extension(config): Extension<Arc<Config>>,
    Json(body): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    if body.wallet_address.len() != 56 || !body.wallet_address.starts_with('G') {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Invalid wallet address" })),
        ));
    }

    // In production: persist user to DB here. Skipped for scaffold.
    let (token, expires_at) = make_token(&body.wallet_address, &config.jwt_secret, config.jwt_expiration)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(AuthResponse {
        token,
        wallet_address: body.wallet_address,
        expires_at,
    }))
}

pub async fn refresh_token(
    Extension(config): Extension<Arc<Config>>,
    Json(body): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<Value>)> {
    let token_data = decode::<Claims>(
        &body.token,
        &DecodingKey::from_secret(config.jwt_secret.as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|_| (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid or expired token" }))))?;

    let wallet = token_data.claims.sub;
    let (new_token, expires_at) = make_token(&wallet, &config.jwt_secret, config.jwt_expiration)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(AuthResponse {
        token: new_token,
        wallet_address: wallet,
        expires_at,
    }))
}
