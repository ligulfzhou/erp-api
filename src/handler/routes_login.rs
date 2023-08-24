use crate::model::account::AccountModel;
use crate::response::api_response::APIEmptyResponse;
use crate::{AppState, ERPError, ERPResult};
use axum::extract::State;
use axum::routing::get;
use axum::{routing::post, Json, Router, middleware};
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use std::sync::Arc;
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use crate::middleware::auth::auth;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/login", post(api_login))
        .route("/api/test", get(api_test).route_layer(middleware::from_fn_with_state(state.clone(), auth)))
        .layer(CookieManagerLayer::new())
        .with_state(state)
}

async fn api_test() -> ERPResult<APIEmptyResponse> {
    Ok(APIEmptyResponse::new())
}
#[derive(Debug, Deserialize, Serialize)]
struct LoginPayload {
    account: String,
    password: String,
}

// #[axum_macros::debug_handler]
async fn api_login(
    cookies: Cookies,
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<LoginPayload>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    tracing::info!("->> {:<12}, api_login", "handler");
    let account = sqlx::query_as::<_, AccountModel>(&format!(
        "select * from accounts where account='{}'",
        payload.account
    ))
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if account.is_none() {
        return Err(ERPError::NotFound("账号不存在".to_string()));
    }

    let account_unwrap = account.unwrap();
    // todo: hash password.
    if account_unwrap.password != payload.password {
        return Err(ERPError::LoginFailForPasswordIsWrong);
    }

    let user_id = account_unwrap.id;
    cookies.add(Cookie::new("user_id", user_id.to_string()));

    Ok(APIEmptyResponse::new())
}

#[cfg(test)]
mod tests {
    use crate::handler::routes_login::LoginPayload;

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let param = LoginPayload {
            account: "test".to_string(),
            password: "test".to_string(),
        };
        let client = httpc_test::new_client("http://localhost:9100")?;
        client
            .do_post("/api/login", serde_json::json!(param))
            .await?
            .print()
            .await?;

        client.do_get("/api/test").await?.print().await?;

        client.do_get("/api/account/info").await?.print().await?;
        Ok(())
    }
}
