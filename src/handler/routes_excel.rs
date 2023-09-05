use crate::constants::STORAGE_FILE_PATH;
// use crate::model::order::{multi_order_items_no_id_models_to_sql, OrderItemNoIdModel};
use crate::excel::excel_order_parser::ExcelOrderParser;
use crate::model::order::OrderModel;
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
        .route("/page/upload", get(page_upload_file))
        .route("/api/upload/excel", post(import_excel))
        .with_state(state)
}

async fn page_upload_file() -> impl IntoResponse {
    Html(
        r#"
<!DOCTYPE html>
<html>
<body>

<form action="/api/upload/excel" method="post" enctype="multipart/form-data">
    Select image to upload:
    <input type="file" name="file" id="fileToUpload">
    <input type="submit" value="Upload Image" name="submit">
</form>

</body>
</html>
    "#,
    )
}

async fn import_excel(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ERPResult<APIEmptyResponse> {
    // let mut id = 0;
    let mut itype = 0;
    let mut file_path: String = "".to_string();

    // 1
    // 这里看着一大堆，其实就是获取三个参数
    // 其中一个参数是 二进制文件，需要保存到本地，并且目录是当前的时间
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
        // } else if name == "id" {
        //     let data = String::from_utf8(field.bytes().await.unwrap().to_vec()).unwrap();
        //     tracing::info!("value of `{}` is: {}", name, data);
        //     id = data
        //         .parse::<i32>()
        //         .map_err(|_| ERPError::ConvertFailed("id".to_string()))?;
        } else if name == "type" {
            let data = String::from_utf8(field.bytes().await.unwrap().to_vec()).unwrap();
            tracing::info!("value of `{}` is: {}", name, data);
            itype = data
                .parse::<i32>()
                .map_err(|_| ERPError::ConvertFailed("type".to_string()))?;
        }
    }

    // 检查文件是否保存成功了
    if file_path.is_empty() {
        return Err(ERPError::Failed("save excel file failed".to_string()));
    }

    // 解析excel文件
    let parser = ExcelOrderParser::new(&file_path, state.db.clone());
    let order_info = parser.parse().await?;
    tracing::info!("order_info: {:#?}", order_info);

    if order_info.exists {
        return Err(ERPError::AlreadyExists("订单已经导入".to_string()));
    }
    Ok(APIEmptyResponse::new())
}
