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

#[derive(Debug, Deserialize)]
struct ImportExcel {
    url: String,
    #[serde(rename(deserialize = "type"))]
    itype: Option<i32>,
    if_order: Option<i32>,
    id: i32,
}

async fn import_excel(
    State(state): State<Arc<AppState>>,
    // WithRejection(Json(payload), _): WithRejection<Json<ImportExcel>, ERPError>,
    mut multipart: Multipart,
) -> ERPResult<APIEmptyResponse> {
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        println!("Length of `{}` is {} bytes", name, data.len());
    }

    println!("state: {:?}", state);
    // println!("{:?}", payload);
    Ok(APIEmptyResponse::new())
}
