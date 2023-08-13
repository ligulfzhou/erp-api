use crate::response::api_response::APIEmptyResponse;
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Multipart, State};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::WithRejection;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/import/order/excel", post(import_order_excel))
        .route("/page/upload/excel", get(page_upload_file))
        .route("/api/upload", post(upload_file))
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

async fn upload_file(mut multipart: Multipart) {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
    }
}

#[derive(Debug, Deserialize)]
struct ImportOrderExcel {
    url: String,
    itype: i32,
}

async fn import_order_excel(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<ImportOrderExcel>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    println!("state: {:?}", state);
    println!("{:?}", payload);
    Ok(APIEmptyResponse::new())
}
