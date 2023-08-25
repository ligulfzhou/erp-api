use crate::dto::dto_account::AccountDto;
use crate::model::account::{AccountModel, DepartmentModel};
use crate::{AppState, ERPError};
use axum::extract::State;
use axum::http::header;
use axum::response::{IntoResponse, Response};
use axum::{routing::post, Json, Router};
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/login", post(api_login))
        .with_state(state)
}

#[derive(Debug, Deserialize, Serialize)]
struct LoginPayload {
    account: String,
    password: String,
}

async fn api_login(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<LoginPayload>, ERPError>,
) -> Result<impl IntoResponse, ERPError> {
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

    let account_id = account_unwrap.id;
    let department = sqlx::query_as::<_, DepartmentModel>(&format!(
        "select * from departments where id={}",
        account_unwrap.department_id
    ))
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    let account_dto = AccountDto::from(account_unwrap, department);

    let cookie = Cookie::build("account_id", account_id.to_string())
        .path("/")
        .max_age(time::Duration::days(7))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    let mut response = Response::new(serde_json::json!(account_dto).to_string());
    response
        .headers_mut()
        .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());

    Ok(response)
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

        client.do_get("/api/account/info").await?.print().await?;
        Ok(())
    }
}
