use crate::AppState;
use axum::extract::Multipart;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
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

// async fn get_goods(
//     State(state): State<Arc<AppState>>,
//     Query(list_goods_param): Query<ListGoodsParam>,
// ) -> ERPResult<APIListResponse<GoodsModel>> {
//     let pagination_sql = list_goods_param.to_pagination_sql();
//     let goods = sqlx::query_as::<_, GoodsModel>(&pagination_sql)
//         .fetch_all(&state.db)
//         .await
//         .map_err(ERPError::DBError)?;
//
//     let count_sql = list_goods_param.to_count_sql();
//     let total: (i64,) = sqlx::query_as(&count_sql)
//         .fetch_one(&state.db)
//         .await
//         .map_err(ERPError::DBError)?;
//
//     Ok(APIListResponse::new(goods, total.0 as i32))
// }
//
