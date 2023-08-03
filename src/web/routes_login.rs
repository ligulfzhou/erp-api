use std::sync::Arc;
use axum::{
    Json, Router,
    routing::post,
};
use serde::Deserialize;
use serde_json::{json, Value};
use crate::{AppState, Error, Result};


#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    pwd: String,
}


async fn api_login(payload: Json<LoginPayload>) -> Result<Json<Value>> {
    println!("->> {:<12}, api_login", "handler");

    if payload.username != "demo1" || payload.pwd != "welcome" {
        return Err(Error::LoginFail);
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
