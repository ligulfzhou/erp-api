use crate::{AppState, ERPError, ERPResult};
use axum::{routing::post, Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

async fn api_login(payload: Json<LoginPayload>) -> ERPResult<Json<Value>> {
    println!("->> {:<12}, api_login", "handler");

    if payload.username != "demo1" || payload.password != "welcome" {
        return Err(ERPError::LoginFail);
    }

    let body = Json(json!({
        "result": {
            "success": true
        }
    }));

    Ok(body)
}

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/login", post(api_login))
        .with_state(state)
}
