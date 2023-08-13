use crate::{AppState, ERPError, ERPResult};
use axum::{routing::post, Json, Router};
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
struct LoginPayload {
    username: String,
    password: String,
}

async fn api_login(
    WithRejection(Json(payload), _): WithRejection<Json<LoginPayload>, ERPError>,
) -> ERPResult<Json<Value>> {
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
