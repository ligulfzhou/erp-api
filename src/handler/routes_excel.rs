use crate::AppState;
use axum::extract::Multipart;
use axum::routing::{get, post};
use axum::{Json, Router};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/upload", post(upload_file))
        // .route("/api/skus", get(get_skus).post(create_skus))
        // .route("/api/sku/update", post(update_sku))
        .with_state(state)
}

async fn upload_file(mut multipart: Multipart) {}

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
