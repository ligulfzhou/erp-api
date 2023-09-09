use crate::{ERPError, ERPResult};
use chrono::NaiveDateTime;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct ProgressModel {
    pub id: i32,            // SERIAL,
    pub order_item_id: i32, // 订单商品ID
    pub step: i32,          // 当前是第几步
    pub account_id: i32,    // 操作人ID
    pub done: bool,         // 完成
    pub notes: String,      // 备注
    pub dt: NaiveDateTime,  // 操作日期
}

pub type OrderItemSteps = HashMap<i32, HashMap<i32, i32>>;

impl ProgressModel {
    pub async fn get_progress_status(
        db: &Pool<Postgres>,
        order_ids: Vec<i32>,
    ) -> ERPResult<OrderItemSteps> {
        // 去获取各产品的流程
        let order_ids_str = order_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let order_item_id_to_order_id = sqlx::query_as::<_, (i32, i32)>(&format!(
            "select id, order_id from order_items where order_id in ({})",
            order_ids_str
        ))
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?
        .into_iter()
        .collect::<HashMap<i32, i32>>();
        tracing::info!("order_item_id => order_id: {:?}", order_item_id_to_order_id);

        let order_item_ids = order_item_id_to_order_id
            .iter()
            .map(|kv| kv.0.clone())
            .collect::<Vec<i32>>();
        if order_item_ids.len() <= 0 {
            return Ok(OrderItemSteps::new());
        }

        let order_item_ids_str = order_item_ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<String>>()
            .join(",");

        let progresses = sqlx::query_as::<_, ProgressModel>(&format!(
            r#"
            select distinct on (order_item_id)
            id, order_item_id, step, account_id, done, notes, dt
            from progress
            where order_item_id in ({})
            order by order_item_id, step desc, id desc;
        "#,
            order_item_ids_str,
        ))
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?;

        tracing::info!("progresses: {:?}", progresses);

        let mut order_item_step = progresses
            .into_iter()
            .map(|progress| {
                if progress.done {
                    (progress.order_item_id, progress.step + 1)
                } else {
                    (progress.order_item_id, progress.step)
                }
            })
            .collect::<HashMap<i32, i32>>();

        tracing::info!("order_item_progress: {:?}", order_item_step);
        for order_item_id in order_item_ids.iter() {
            order_item_step.entry(order_item_id.to_owned()).or_insert(1);
        }

        tracing::info!("after order_item_progress: {:?}", order_item_step);
        let mut order_items_steps = OrderItemSteps::new();
        for order_id in order_ids.iter() {
            let mut order_item_progress_stats = HashMap::new();
            for (order_item_id, step) in order_item_step.iter() {
                let order_id_ = order_item_id_to_order_id.get(order_item_id).unwrap_or(&0);
                if order_id_ == order_id {
                    let count = order_item_progress_stats.entry(step.clone()).or_insert(0);
                    *count += 1;
                }
            }

            order_items_steps.insert(order_id.clone(), order_item_progress_stats);
        }

        Ok(order_items_steps)
    }
}
