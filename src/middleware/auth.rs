use crate::dto::dto_account::AccountDto;
use crate::model::account::{AccountModel, DepartmentModel};
use crate::{AppState, ERPError};
use axum::{extract::State, http::Request, middleware::Next, response::IntoResponse};
use axum_extra::extract::cookie::CookieJar;
use serde::Serialize;
use std::sync::Arc;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

pub async fn auth<B>(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState>>,
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, ERPError> {
    let account_id = cookie_jar
        .get("account_id")
        .map(|cookie| cookie.value().to_string())
        .ok_or(ERPError::NotAuthorized)?;

    let account_id = account_id.parse::<i32>().unwrap_or(0);
    tracing::info!("account_id: {}", account_id);

    let account = sqlx::query_as!(
        AccountModel,
        "select * from accounts where id = $1",
        account_id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .ok_or(ERPError::NotFound("账号不存在".to_string()))?;

    let department = sqlx::query_as::<_, DepartmentModel>(&format!(
        "select * from departments where id={}",
        account.department_id
    ))
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .ok_or(ERPError::NotFound("账号不存在".to_string()))?;

    let account_dto = AccountDto::from(account, department);

    req.extensions_mut().insert(account_dto);
    Ok(next.run(req).await)
}
