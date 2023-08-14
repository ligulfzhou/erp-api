use crate::constants::STORAGE_FILE_PATH;
use crate::excel::order_umya_excel::read_excel_with_umya;
use crate::response::api_response::APIEmptyResponse;
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Multipart, State};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use chrono::{Datelike, Timelike, Utc};
use std::fs;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/page/upload/excel", get(page_upload_file))
        .route("/api/upload/excel", post(import_excel))
        .with_state(state)
}

async fn page_upload_file() -> impl IntoResponse {
    Html(
        r#"
<!DOCTYPE html>
<html>
<body>

<form action="/api/upload" method="post" enctype="multipart/form-data">
    Select image to upload:
    <input type="file" name="fileToUpload" id="fileToUpload">
    <input type="submit" value="Upload Image" name="submit">
</form>

</body>
</html>
    "#,
    )
}

// #[derive(Debug, Deserialize)]
// struct ImportExcel {
//     url: String,
//     #[serde(rename(deserialize = "type"))]
//     itype: Option<i32>,
//     if_order: Option<i32>,
//     id: i32,
// }

async fn import_excel(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ERPResult<APIEmptyResponse> {
    let mut id = 0;
    let mut itype = 0;
    let mut file_path: String = "".to_string();
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name == "file" {
            let data = field.bytes().await.unwrap();
            let now = Utc::now();
            let dir_path = format!(
                "{}{}{:02}{:02}",
                STORAGE_FILE_PATH,
                now.year(),
                now.month(),
                now.day()
            );
            let file_name = format!(
                "{}{:02}{:02}{:02}{:02}{:02}.xlsx",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second()
            );
            fs::create_dir_all(&dir_path)
                .map_err(|_| ERPError::SaveFileFailed(format!("create {} failed", dir_path)))?;
            tracing::info!("Length of `{}` is {} bytes", name, data.len());
            let file_path_full = format!("{}/{}", dir_path, file_name);
            fs::write(&file_path_full, data).map_err(|_| {
                ERPError::SaveFileFailed(format!("create {} failed", file_path_full))
            })?;
            file_path = file_path_full;
        } else if name == "id" {
            let data = String::from_utf8(field.bytes().await.unwrap().to_vec()).unwrap();
            tracing::info!("value of `{}` is: {}", name, data);
            id = data
                .parse::<i32>()
                .map_err(|_| ERPError::ConvertFailed("id".to_string()))?;
        } else if name == "type" {
            let data = String::from_utf8(field.bytes().await.unwrap().to_vec()).unwrap();
            tracing::info!("value of `{}` is: {}", name, data);
            itype = data
                .parse::<i32>()
                .map_err(|_| ERPError::ConvertFailed("type".to_string()))?;
        }
    }

    if file_path.is_empty() {
        return Err(ERPError::Failed("save excel file failed".to_string()));
    }

    let items = read_excel_with_umya(&file_path);
    tracing::info!("we extract order_items from excel: {:?}", items);

    Ok(APIEmptyResponse::new())
}
