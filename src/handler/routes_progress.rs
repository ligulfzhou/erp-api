use crate::constants::DONE_INDEX;
use crate::dto::dto_account::AccountDto;
use crate::dto::dto_orders::{
    OrderGoodsDto, OrderGoodsItemDto, OrderGoodsItemWithStepsDto,
    OrderGoodsWithStepsWithItemStepDto,
};
use crate::dto::dto_progress::OneProgress;
use crate::middleware::auth::auth;
use crate::model::goods::{GoodsModel, SKUModel};
use crate::model::order::OrderModel;
use crate::model::progress::ProgressModel;
use crate::response::api_response::{APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{middleware, Json};
use axum::{Extension, Router};
use axum_extra::extract::WithRejection;
use chrono::Utc;
use itertools::Itertools;
use std::collections::HashMap;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/revoke/progress", post(revoke_progress))
        .route("/api/mark/progress", post(mark_progress))
        .route(
            "/api/get/order/item/progress",
            get(get_order_items_progress),
        )
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state)
}

#[derive(Deserialize)]
struct OrderItemProgressParam {
    order_no: String,
    goods_no: String,
}

async fn get_order_items_progress(
    Extension(account): Extension<AccountDto>,
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<OrderItemProgressParam>, ERPError>,
) -> ERPResult<APIListResponse<OrderGoodsWithStepsWithItemStepDto>> {
    let order_id = sqlx::query_as!(
        OrderModel,
        "select * from orders where order_no = $1",
        param.order_no
    )
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .ok_or(ERPError::ParamError("订单号未找到".to_string()))?
    .id;

    let goods = sqlx::query_as!(
        GoodsModel,
        "select * from goods where goods_no=$1",
        param.goods_no
    )
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    let mut goods_id: i32 = 0;

    let item_ids = match goods {
        Some(real_goods) => {
            goods_id = real_goods.id;

            sqlx::query!("select id from skus where goods_id = $1", real_goods.id)
                .fetch_all(&state.db)
                .await
                .map_err(ERPError::DBError)?
                .into_iter()
                .map(|r| r.id)
                .collect::<Vec<i32>>()
        }
        None => {
            let sku = sqlx::query_as!(
                SKUModel,
                "select * from skus where sku_no = $1",
                param.goods_no
            )
            .fetch_optional(&state.db)
            .await
            .map_err(ERPError::DBError)?
            .ok_or(ERPError::ParamError("商品编号未找到".to_string()))?;

            goods_id = sku.goods_id;

            vec![sku.id]
        }
    };

    // 获取order_good
    let order_goods: Vec<OrderGoodsDto> = sqlx::query_as!(
        OrderGoodsDto,
        r#"
        select
            og.id as id, og.order_id as order_id, og.goods_id as goods_id, g.goods_no as goods_no,
            g.name as name, og.images as images, og.image_des as image_des, g.plating as plating,
            og.package_card as package_card, og.package_card_des as package_card_des
        from order_goods og, goods g
        where og.goods_id = g.id and og.order_id = $1
            and g.id = $2
        order by og.id;
        "#,
        order_id,
        goods_id,
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if order_goods.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }
    // tracing::info!("order_goods: {:?}, len: {}", order_goods, order_goods.len());

    let order_goods_ids = order_goods.iter().map(|item| item.id).collect::<Vec<i32>>();
    tracing::info!("order_goods_ids: {:?}", order_goods_ids);

    // 用order_goods_ids去获取order_items
    let order_items_dto: Vec<OrderGoodsItemDto> = sqlx::query_as!(
        OrderGoodsItemDto,
        r#"
        select
            oi.id, oi.order_id, oi.sku_id, s.color, s.sku_no, oi.count, oi.unit,
            oi.unit_price, oi.total_price, oi.notes, og.goods_id, oi.order_goods_id,
            oi.notes_images
        from order_items oi, skus s, order_goods og
        where oi.sku_id = s.id and oi.order_goods_id = og.id
            and oi.order_goods_id = any($1)
        order by id;
        "#,
        &order_goods_ids
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if order_items_dto.is_empty() {
        return Ok(APIListResponse::new(
            order_goods
                .iter()
                .map(|item| {
                    OrderGoodsWithStepsWithItemStepDto::from_order_with_goods_and_steps_and_items(
                        item.clone(),
                        HashMap::new(),
                        vec![],
                        false,
                        0,
                        0,
                    )
                })
                .collect::<Vec<OrderGoodsWithStepsWithItemStepDto>>(),
            order_goods.len() as i32,
        ));
    }
    let order_item_ids = order_items_dto
        .iter()
        .map(|item| item.id)
        .collect::<Vec<i32>>();
    tracing::info!("order_item_ids: {:?}", order_item_ids);

    // 获取所有的order_item的流程数据
    let progresses = sqlx::query_as!(
        OneProgress,
        r#"
        select
            p.*, a.name as account_name, d.name as department
        from progress p, accounts a, departments d
        where p.account_id = a.id and a.department_id = d.id
            and p.order_item_id = any($1)
        order by p.id;
        "#,
        &order_item_ids
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;
    tracing::info!("progresses: {:?}", progresses);

    let mut order_item_id_to_progress_vec = HashMap::new();
    progresses.into_iter().for_each(|one_progress| {
        let progress_vec = order_item_id_to_progress_vec
            .entry(one_progress.order_item_id)
            .or_insert(vec![]);
        progress_vec.push(one_progress);
    });

    let empty: Vec<OneProgress> = vec![];
    let order_items_with_steps_dtos = order_items_dto
        .into_iter()
        .map(|item| {
            let steps = order_item_id_to_progress_vec
                .get(&item.id)
                .unwrap_or(&empty);

            let step_for_checking_next_action = {
                match steps.len() {
                    0 => 1,
                    _ => match steps[steps.len() - 1].done {
                        true => steps[steps.len() - 1].step + 1,
                        false => steps[steps.len() - 1].step,
                    },
                }
            };
            let is_next_action = account.steps.contains(&step_for_checking_next_action);

            OrderGoodsItemWithStepsDto::from(
                item,
                steps.clone(),
                is_next_action,
                step_for_checking_next_action,
            )
        })
        .collect::<Vec<OrderGoodsItemWithStepsDto>>();

    let mut ogid_to_order_items_dto = HashMap::new();
    order_items_with_steps_dtos
        .clone()
        .into_iter()
        .for_each(|item| {
            let dtos = ogid_to_order_items_dto
                .entry(item.order_goods_id)
                .or_insert(vec![]);
            dtos.push(item);
        });

    let empty_array: Vec<OrderGoodsItemWithStepsDto> = vec![];
    let order_goods_dtos = order_goods
        .into_iter()
        .map(|order_good| {
            let items = ogid_to_order_items_dto
                .get(&order_good.id)
                .unwrap_or(&empty_array);

            let order_item_to_step_index = items
                .iter()
                .map(|item| {
                    let step_index = match &item.steps.len() {
                        0 => (1, 0),
                        _ => (
                            item.steps[item.steps.len() - 1].step,
                            item.steps[item.steps.len() - 1].index,
                        ),
                    };
                    (item.id, step_index)
                })
                .collect::<HashMap<i32, (i32, i32)>>();

            let mut order_item_to_step_index_count: HashMap<(i32, i32), i32> = HashMap::new();
            order_item_to_step_index.iter().for_each(|(_, step)| {
                let count = order_item_to_step_index_count
                    .entry(step.to_owned())
                    .or_insert(0);
                *count += 1;
            });

            let mut is_next_action = false;
            let mut current_step = 0;

            // 如果这个款式下的进度一样，才能做一起做
            let step_indexs = order_item_to_step_index_count
                .iter()
                .map(|sc| sc.0.to_owned())
                .collect::<Vec<(i32, i32)>>();
            if step_indexs.len() == 1 {
                if step_indexs[0].1 == 2 {
                    current_step = step_indexs[0].0 + 1;
                } else {
                    current_step = step_indexs[0].0;
                }
                if account.steps.contains(&current_step) {
                    is_next_action = true;
                }
            }
            // println!("steps: {:?}, {}", steps, is_next_action);
            OrderGoodsWithStepsWithItemStepDto::from_order_with_goods_and_steps_and_items(
                order_good,
                order_item_to_step_index_count,
                items.clone(),
                is_next_action,
                current_step,
                0,
            )
        })
        .collect::<Vec<OrderGoodsWithStepsWithItemStepDto>>();

    let count = sqlx::query!(
        "select count(1) from order_goods where order_id = $1",
        order_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .count
    .unwrap_or(0) as i32;

    Ok(APIListResponse::new(order_goods_dtos, count))
}

async fn get_order_items_progress_bk(
    // Extension(account): Extension<AccountDto>,
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<OrderItemProgressParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    // 检查订单号是否存在
    let order_id = sqlx::query_as!(
        OrderModel,
        "select * from orders where order_no = $1",
        param.order_no
    )
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .ok_or(ERPError::ParamError("订单号未找到".to_string()))?
    .id;

    // 先检查这个goods_no是: 商品编号/sku编号
    let goods = sqlx::query_as!(
        GoodsModel,
        "select * from goods where goods_no=$1",
        param.goods_no
    )
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    let item_ids = match goods {
        Some(real_goods) => sqlx::query!("select id from skus where goods_id = $1", real_goods.id)
            .fetch_all(&state.db)
            .await
            .map_err(ERPError::DBError)?
            .into_iter()
            .map(|r| r.id)
            .collect::<Vec<i32>>(),
        None => {
            vec![
                sqlx::query_as!(
                    SKUModel,
                    "select * from skus where sku_no = $1",
                    param.goods_no
                )
                .fetch_optional(&state.db)
                .await
                .map_err(ERPError::DBError)?
                .ok_or(ERPError::ParamError("商品编号未找到".to_string()))?
                .id,
            ]
        }
    };

    Ok(APIEmptyResponse::new())
}

#[derive(Deserialize)]
struct RevokeProgressParam {
    id: i32,
}

async fn revoke_progress(
    Extension(account): Extension<AccountDto>,
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<RevokeProgressParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let progress = sqlx::query_as!(
        ProgressModel,
        "select * from progress where id = $1",
        payload.id
    )
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .ok_or(ERPError::NotFound("该流程不存在".to_string()))?;

    if !account.steps.contains(&progress.step) {
        return Err(ERPError::NoPermission("无操作权限".to_string()));
    }

    sqlx::query!("delete from progress where id = $1", payload.id)
        .execute(&state.db)
        .await
        .map_err(|_| ERPError::Failed("删除数据失败".to_string()))?;

    Ok(APIEmptyResponse::new())
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

        let order_item_ids = sqlx::query!(
            "select id from order_items where order_goods_id=$1",
            order_goods_id
        )
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?
        .into_iter()
        .map(|r| r.id)
        .collect::<Vec<i32>>();

        if order_item_ids.is_empty() {
            return Err(ERPError::NotFound(
                "该商品下无添加任何颜色/款式".to_string(),
            ));
        }

        let progresses = sqlx::query_as!(
            ProgressModel,
            r#"
            select distinct on (order_item_id)
            id, order_item_id, step, account_id, done, notes, dt, index
            from progress
            where order_item_id = any($1)
            order by order_item_id, step, id desc;
            "#,
            &order_item_ids,
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
        order_item_ids.iter().for_each(|order_item_id| {
            order_item_progress
                .entry(order_item_id.to_owned())
                .or_insert(1);
        });
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
        let to_insert_progress_models = order_item_ids
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
