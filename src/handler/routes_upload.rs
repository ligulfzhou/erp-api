use crate::constants::STORAGE_FILE_PATH;
use crate::constants::STORAGE_URL_PREFIX;
use crate::response::api_response::APIDataResponse;
use crate::{AppState, ERPResult, ERPError};
use axum::extract::{Multipart, State};
use axum::routing::post;
use axum::Router;
use chrono::{Datelike, Timelike, Utc};
use std::sync::Arc;
use std::fs;


pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/upload/image", post(upload_image))
        .with_state(state)
}

#[derive(Debug, Serialize)]
struct ImageUrlResponse {
    url: String,
}
async fn upload_image(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ERPResult<APIDataResponse<ImageUrlResponse>> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        tracing::info!("field name: {}", name);
        if name == "file" {
            let data = field.bytes().await.unwrap();
            let now = Utc::now();
            let dir_path = format!(
                "{}images/{}{:02}{:02}",
                STORAGE_FILE_PATH,
                now.year(),
                now.month(),
                now.day()
            );
            tracing::info!("dir_path: {}", dir_path);
            let file_name = format!(
                "{}{:02}{:02}{:02}{:02}{:02}.png",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second()
            );
            tracing::info!("filename: {}", file_name);
            fs::create_dir_all(&dir_path)
                .map_err(|_| ERPError::SaveFileFailed(format!("create {} failed", dir_path)))?;
            tracing::info!("Length of `{}` is {} bytes", name, data.len());
            let file_path_full = format!("{}/{}", dir_path, file_name);
            fs::write(&file_path_full, data).map_err(|_| {
                ERPError::SaveFileFailed(format!("create {} failed", file_path_full))
            })?;

            let url = format!("{}images/{}", STORAGE_URL_PREFIX, file_name);
            return Ok(APIDataResponse::new(ImageUrlResponse{url}))
        }
    }

    Err(ERPError::SaveFileFailed("文件存储失败".to_string()))
}
