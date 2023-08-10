use axum::response::Html;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::model::item::ItemModel;
use crate::AppState;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/items", get(handler_get_item_list))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct ListParam {
    num: Option<String>,
    memo: Option<String>,
    page: Option<i32>,
    page_size: Option<i32>,
}

async fn handler_get_item_list(
    Query(list_param): Query<ListParam>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let page = list_param.page.unwrap_or(1);
    let page_size = list_param.page_size.unwrap_or(20);

    let limit = page_size;
    let offset = (page - 1) * page_size;

    Html("")
    // match sqlx::query_as!(
    //     ItemModel,
    //     "select * from orders order by id desc limit $1 offset $2",
    //     limit as i64,
    //     offset as i64
    // )
    // .fetch_all(&state.db)
    // .await
    // {
    //     Ok(query_result) => {
    //         let json_response = serde_json::json!({
    //             "status": "success",
    //             "results": query_result.len(),
    //             "item": query_result
    //         });
    //         Ok(Json(json_response))
    //     }
    //     Err(err) => {
    //         let error_response = serde_json::json!({
    //             "status": "fail",
    //             "message": "Something bad happened while fetching all note items",
    //         });
    //         Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)))
    //     }
    // }
}
