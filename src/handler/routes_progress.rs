use crate::dto::dto_account::AccountDto;
use crate::middleware::auth::auth;
use crate::model::order::OrderItemModel;
use crate::model::progress::ProgressModel;
use crate::response::api_response::APIEmptyResponse;
use crate::{AppState, ERPError, ERPResult};
use axum::extract::State;
use axum::routing::post;
use axum::{middleware, Json};
use axum::{Extension, Router};
use axum_extra::extract::WithRejection;
use std::fs::read;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/mark/progress", post(mark_progress))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state)
}

#[derive(Debug, Deserialize, Serialize)]
struct MarkProgressParam {
    order_goods_id: Option<i32>,
    order_item_id: Option<i32>,
}

async fn mark_progress(
    Extension(account): Extension<AccountDto>,
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<MarkProgressParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    // 1: 先获得这个产品当前的状态
    let order_goods_id = payload.order_goods_id.unwrap_or(0);
    let order_item_id = payload.order_item_id.unwrap_or(0);
    if order_item_id == 0 && order_goods_id == 0 {
        return Err(ERPError::ParamNeeded(
            "order_goods_id/order_item_id".to_string(),
        ));
    }

    if order_goods_id > 0 {
    } else {
        // 获得上一个 节点 在什么步骤
        let progress = sqlx::query_as::<_, ProgressModel>(&format!(
            "select * from progress where order_item_id={order_item_id} order by index, id desc limit 1"
        ))
        .fetch_optional(&state.db)
        .await
        .map_err(ERPError::DBError)?;

        let index = match progress {
            None => 0,
            Some(real) => {
                if real.done {
                    real.step + 1
                } else {
                    real.step
                }
            }
        };
        // if account
    }

    Ok(APIEmptyResponse::new())
}
