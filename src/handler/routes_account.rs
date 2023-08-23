use crate::{AppState, ERPResult};
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;
use tower_cookies::{cookie, CookieManagerLayer, Cookies};

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/account/info", get(account_info))
        .layer(CookieManagerLayer::new())
        .with_state(state)
}

// todo
async fn account_info(cookies: Cookies) -> ERPResult<Json<Value>> {
    if let Some(user_id) = cookies.get("user_id") {
        tracing::info!("{}", user_id.value());
    }
    Ok(Json(json!({})))
}
