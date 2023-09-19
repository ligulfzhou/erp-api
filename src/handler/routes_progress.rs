use crate::constants::DONE_INDEX;
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
use std::collections::HashMap;
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
    index: i32,
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
    // let done = payload.done.unwrap_or(false);
    // let notes = payload.notes.as_deref().unwrap_or("");
    // if !done && notes.is_empty() {
    //     return Err(ERPError::ParamError("done和notes，至少要有一样".to_owned()));
    // }

    if payload.index == 0 {
        return Err(ERPError::ParamError("请选择正确的流程".to_string()));
    }

    if order_goods_id > 0 {
        let order_goods = sqlx::query_as::<_, (i32,)>(&format!(
            "select id from order_goods where id = {order_goods_id}"
        ))
        .fetch_optional(&state.db)
        .await
        .map_err(ERPError::DBError)?;
        if order_goods.is_none() {
            return Err(ERPError::NotFound("订单商品不存在".to_string()));
        }

        // let (order_id, goods_id) = order_goods.unwrap();
        // tracing::info!("order_id: {order_id}, goods_id: {goods_id}");

        let order_item_ids = sqlx::query_as::<_, (i32,)>(&format!(
            "select id from order_items where order_goods_id={}",
            order_goods_id
        ))
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

        tracing::info!("order_item_ids: {:?}", order_item_ids);
        if order_item_ids.is_empty() {
            return Err(ERPError::NotFound(
                "该商品下无添加任何颜色/款式".to_string(),
            ));
        }

        let order_item_ids_vec = order_item_ids
            .into_iter()
            .map(|item| item.0)
            .collect::<Vec<i32>>();

        let progresses = sqlx::query_as!(
            ProgressModel,
            r#"
            select distinct on (order_item_id)
            id, order_item_id, step, account_id, done, notes, dt, index
            from progress
            where order_item_id = any($1)
            order by order_item_id, step, id desc;
            "#,
            &order_item_ids_vec,
        )
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;
        // tracing::info!("progresses: {:?}", progresses);

        let mut order_item_progress = progresses
            .into_iter()
            .map(|progress| {
                if progress.done {
                    (progress.order_item_id, progress.step + 1)
                } else {
                    (progress.order_item_id, progress.step)
                }
            })
            .collect::<HashMap<i32, i32>>();

        tracing::info!("order_item_progress: {:?}", order_item_progress);
        for order_item_id in order_item_ids_vec.iter() {
            order_item_progress
                .entry(order_item_id.to_owned())
                .or_insert(1);
        }
        tracing::info!("after order_item_progress: {:?}", order_item_progress);
        // 检查所有的产品，是否在同一个步骤上
        let mut values = order_item_progress
            .into_iter()
            .map(|oip| oip.1)
            .collect::<Vec<i32>>();

        tracing::info!("order_item_progress values: {:?}", values);
        values.dedup();

        tracing::info!("after dedup order_item_progress values: {:?}", values);
        if values.len() > 1 {
            return Err(ERPError::Failed(
                "该产品的所有颜色，不在同一个流程下，请单独处理".to_string(),
            ));
        }

        let step = values[0];
        if !account.steps.contains(&step) {
            return Err(ERPError::NoPermission(
                "当前的状态并不是你可以修改的".to_string(),
            ));
        }

        let now = Utc::now();
        // let now_str = format_datetime(now);
        let to_insert_progress_models = order_item_ids_vec
            .iter()
            .map(|oii| ProgressModel {
                id: 0,
                order_item_id: *oii,
                step,
                index: payload.index,
                account_id: account.id,
                done: payload.index == DONE_INDEX,
                notes: payload.notes.clone(),
                dt: now,
            })
            .collect::<Vec<ProgressModel>>();
        ProgressModel::insert_multiple(&state.db, &to_insert_progress_models).await?;
    } else {
        // 获得上一个 节点 在什么步骤
        let order_item = sqlx::query_as::<_, (i32,)>(&format!(
            "select id from order_items where id = {order_item_id}"
        ))
        .fetch_optional(&state.db)
        .await
        .map_err(ERPError::DBError)?;
        if order_item.is_none() {
            return Err(ERPError::NotFound("订单商品不存在".to_string()));
        }

        let progress = sqlx::query_as!(
            ProgressModel,
            "select * from progress where order_item_id=$1 order by step, id desc limit 1",
            order_item_id
        )
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

        let now = Utc::now();
        sqlx::query!(
            r#"
            insert into progress (order_item_id, step, index, account_id, done, notes, dt)
            values ($1, $2, $3, $4, $5, $6, $7)
            "#,
            order_item_id,
            step,
            payload.index,
            account.id,
            DONE_INDEX == payload.index,
            payload.notes,
            now
        )
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
    async fn test_mark_progress_on_order_items() -> anyhow::Result<()> {
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
            notes: "notes..".to_string(),
            index: 1,
        };
        client
            .do_post("/api/mark/progress", serde_json::json!(param))
            .await?
            .print()
            .await?;

        let param = MarkProgressParam {
            order_goods_id: None,
            order_item_id: Some(1),
            index: 2,
            notes: "".to_string(),
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

    #[tokio::test]
    async fn test_mark_progress_on_order_goods() -> anyhow::Result<()> {
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
            order_goods_id: Some(1),
            order_item_id: None,
            index: 1,
            notes: "notes..".to_string(),
        };
        client
            .do_post("/api/mark/progress", serde_json::json!(param))
            .await?
            .print()
            .await?;

        let param = MarkProgressParam {
            order_goods_id: Some(1),
            order_item_id: None,
            index: 2,
            notes: "".to_string(),
        };
        client
            .do_post("/api/mark/progress", serde_json::json!(param))
            .await?
            .print()
            .await?;

        let param = MarkProgressParam {
            order_goods_id: Some(3),
            order_item_id: None,
            index: 2,
            notes: "".to_string(),
        };
        client
            .do_post("/api/mark/progress", serde_json::json!(param))
            .await?
            .print()
            .await?;
        Ok(())
    }
}
