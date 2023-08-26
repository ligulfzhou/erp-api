use axum::extract::rejection::JsonRejection;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::error::Error as SqlxError;
use thiserror::Error;

pub type ERPResult<T> = Result<T, ERPError>;

#[derive(Debug, Error)]
pub enum ERPError {
    #[error("登陆失败")]
    LoginFail,

    #[error("密码错误")]
    LoginFailForPasswordIsWrong,

    #[error("未登陆")]
    NotAuthorized,

    #[error("账号不存在")]
    AccountNotFound,

    #[error("sqlx db error: {:?}", .0)]
    DBError(#[from] SqlxError),

    #[error("数据已存在: {:?}", .0)]
    AlreadyExists(String),

    #[error("数据未找到: {:?}", .0)]
    NotFound(String),

    #[error("参数缺失: {:?}", .0)]
    ParamNeeded(String),

    #[error(transparent)]
    JsonExtractorRejection(#[from] JsonRejection),

    #[error("{}", .0)]
    SaveFileFailed(String),

    #[error("Param type conversion Failed: {:?}", .0)]
    ConvertFailed(String),

    #[error("{}", .0)]
    Failed(String),

    #[error("collision: {}", .0)]
    Collision(String),
}

impl IntoResponse for ERPError {
    fn into_response(self) -> Response {
        print!("->> {:<12} - {self:?}", "INTO_RES");

        let msg = self.to_string();

        (
            StatusCode::OK,
            serde_json::json!({
                "code": 1, // failed code is always 1
                "msg": msg
            })
            .to_string(),
        )
            .into_response()
    }
}
