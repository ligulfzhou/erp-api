use crate::common::datetime::format_datetime;
use crate::dto::dto_account::AccountDto;
use crate::middleware::auth::auth;
use crate::model::progress::ProgressModel;
use crate::response::api_response::APIEmptyResponse;
use crate::{AppState, ERPError, ERPResult};
use axum::extract::State;
use axum::routing::post;
use axum::{middleware, Json};
use axum::{Extension, Router};
use axum_extra::extract::WithRejection;
use chrono::Utc;
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
    done: bool,
    notes: String,
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

        let order_item = sqlx::query_as::<_, (i32,)>(&format!(
            "select * from order_items where id = {order_item_id}"
        ))
        .fetch_optional(&state.db)
        .await
        .map_err(ERPError::DBError)?;
        if order_item.is_none() {
            return Err(ERPError::NotFound("订单商品不存在".to_string()));
        }

        let progress = sqlx::query_as::<_, ProgressModel>(&format!(
            "select * from progress where order_item_id={order_item_id} order by step, id desc limit 1"
        ))
        .fetch_optional(&state.db)
        .await
        .map_err(ERPError::DBError)?;

        let step = match progress {
            None => 1,
            Some(real) => {
                if real.done {
                    real.step + 1
                } else {
                    real.step
                }
            }
        };
        if !account.steps.contains(&step) {
            return Err(ERPError::NoPermission(
                "当前的状态并不是你可以修改的".to_string(),
            ));
        }

        let now = Utc::now().naive_utc();
        sqlx::query(&format!(
            r#"
            insert into progress (order_item_id, step, account_id, done, notes, dt) 
            values ({}, {}, {}, {}, '{}', '{}')
            "#,
            order_item_id,
            step,
            account.id,
            payload.done,
            payload.notes,
            format_datetime(now)
        ))
        .execute(&state.db)
        .await
        .map_err(ERPError::DBError)?;
    }

    Ok(APIEmptyResponse::new())
}

#[cfg(test)]
mod tests {
    use crate::handler::routes_login::LoginPayload;
    use crate::handler::routes_progress::MarkProgressParam;

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let param = LoginPayload {
            account: "test".to_string(),
            password: "test".to_string(),
        };
        let client = httpc_test::new_client("http://localhost:9100")?;
        client
            .do_post("/api/login", serde_json::json!(param))
            .await?
            .print()
            .await?;

        let param = MarkProgressParam {
            order_goods_id: None,
            order_item_id: Some(1),
            done: false,
            notes: "notes..".to_string(),
        };
        client
            .do_post("/api/mark/progress", serde_json::json!(param))
            .await?
            .print()
            .await?;

        let param = MarkProgressParam {
            order_goods_id: None,
            order_item_id: Some(1),
            done: true,
            notes: "notes..".to_string(),
        };
        client
            .do_post("/api/mark/progress", serde_json::json!(param))
            .await?
            .print()
            .await?;

        client
            .do_post("/api/mark/progress", serde_json::json!(param))
            .await?
            .print()
            .await?;

        Ok(())
    }
}
