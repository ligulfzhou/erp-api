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
    let user_id = cookie_jar
        .get("user_id")
        .map(|cookie| cookie.value().to_string());
    if user_id.is_none() {
        return Err(ERPError::NotAuthorized);
    }

    let user_id = user_id.unwrap();

    let user =
        sqlx::query_as::<_, AccountModel>(&format!("select * from accounts where id={}", user_id))
            .fetch_optional(&state.db)
            .await
            .map_err(ERPError::DBError)?;

    if user.is_none() {
        return Err(ERPError::AccountNotFound);
    }

    let user = user.unwrap();
    let department = sqlx::query_as::<_, DepartmentModel>(&format!(
        "select * from departments where id={}",
        user.department_id
    ))
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    let user_dto = AccountDto::from(user, department);

    req.extensions_mut().insert(user_dto);
    Ok(next.run(req).await)
}
