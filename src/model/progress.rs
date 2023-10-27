use crate::{ERPError, ERPResult};
use chrono::{DateTime, Utc};
use sqlx::{Pool, Postgres, QueryBuilder};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct ProgressModel {
    pub id: i32,            // SERIAL,
    pub order_item_id: i32, // 订单商品ID
    pub step: i32,          // 当前是第几步
    pub index: i32,         // 部门内的具体某流程
    pub account_id: i32,    // 操作人ID
    pub done: bool,         // 完成
    pub notes: String,      // 备注
    pub dt: DateTime<Utc>,  // 操作日期
}

// order_id, (step, index), count
pub type OrderItemSteps = HashMap<i32, HashMap<(i32, i32), i32>>;

#[derive(sqlx::FromRow)]
struct IdId {
    id: i32,
    order_id: i32,
}

impl ProgressModel {
    pub async fn insert_multiple(
        db: &Pool<Postgres>,
        rows: &[ProgressModel],
    ) -> ERPResult<Vec<ProgressModel>> {
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "insert into progress (order_item_id, step, index, account_id, done, notes, dt) ",
        );

        query_builder.push_values(rows, |mut b, item| {
            b.push_bind(item.order_item_id)
                .push_bind(item.step)
                .push_bind(item.index)
                .push_bind(item.account_id)
                .push_bind(item.done)
                .push_bind(item.notes.clone())
                .push_bind(item.dt);
        });
        query_builder.push(" returning *;");

        let res = query_builder
            .build_query_as::<ProgressModel>()
            .fetch_all(db)
            .await
            .map_err(ERPError::DBError)?;

        Ok(res)
    }

    pub async fn get_order_total_count(
        db: &Pool<Postgres>,
        order_ids: &[i32],
    ) -> ERPResult<HashMap<i32, i32>> {
        let order_id_to_exception_count = sqlx::query!(
            r#"
            select o.id, count(1)
            from orders o, order_items oi
            where o.id = oi.order_id
                and o.id = any($1)
            group by o.id;
            "#,
            order_ids
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?
        .into_iter()
        .map(|r| (r.id, r.count.unwrap_or(0) as i32))
        .collect::<HashMap<i32, i32>>();

        Ok(order_id_to_exception_count)
    }

    pub async fn get_order_done_count(
        db: &Pool<Postgres>,
        order_ids: &[i32],
    ) -> ERPResult<HashMap<i32, i32>> {
        let order_id_to_exception_count = sqlx::query!(
            r#"
            select o.id, count(1)
            from orders o, order_items oi, progress p
            where o.id = oi.order_id and p.order_item_id=oi.id
                and o.id = any($1) and p.step=7 and p.index=2
            group by o.id;
            "#,
            order_ids
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?
        .into_iter()
        .map(|r| (r.id, r.count.unwrap_or(0) as i32))
        .collect::<HashMap<i32, i32>>();

        Ok(order_id_to_exception_count)
    }

    pub async fn get_order_exception_count(
        db: &Pool<Postgres>,
        order_ids: &[i32],
    ) -> ERPResult<HashMap<i32, i32>> {
        let order_id_to_exception_count = sqlx::query!(
            r#"
            select o.id, count(1)
            from orders o, order_items oi, progress p
            where o.id = oi.order_id and p.order_item_id=oi.id
                and o.id = any($1) and p.index=1
            group by o.id;
            "#,
            order_ids
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?
        .into_iter()
        .map(|r| (r.id, r.count.unwrap_or(0) as i32))
        .collect::<HashMap<i32, i32>>();

        Ok(order_id_to_exception_count)
    }

    pub async fn get_progress_status(
        db: &Pool<Postgres>,
        order_ids: &[i32],
    ) -> ERPResult<OrderItemSteps> {
        // 去获取各产品的流程
        let order_item_id_to_order_id = sqlx::query_as!(
            IdId,
            "select id, order_id from order_items where order_id = any($1)",
            order_ids
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?
        .into_iter()
        .map(|idid| (idid.id, idid.order_id))
        .collect::<HashMap<i32, i32>>();

        let order_item_ids = order_item_id_to_order_id
            .iter()
            .map(|kv| *kv.0)
            .collect::<Vec<i32>>();
        if order_item_ids.is_empty() {
            return Ok(OrderItemSteps::new());
        }

        let progresses = sqlx::query_as!(
            ProgressModel,
            r#"
            select distinct on (order_item_id)
            id, order_item_id, step, account_id, done, notes, dt, index
            from progress
            where order_item_id = any($1)
            order by order_item_id, step desc, id desc;
            "#,
            &order_item_ids,
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?;

        // tracing::info!("progresses: {:?}", progresses);

        let mut order_item_step = progresses
            .into_iter()
            .map(|progress| {
                (progress.order_item_id, (progress.step, progress.index))
                // if progress.done {
                //     (progress.order_item_id, progress.step + 1)
                // } else {
                //     (progress.order_item_id, progress.step)
                // }
            })
            .collect::<HashMap<i32, (i32, i32)>>();

        tracing::info!("order_item_step: {:?}", order_item_step);
        order_item_ids.iter().for_each(|order_item_id| {
            order_item_step
                .entry(order_item_id.to_owned())
                .or_insert((1, 0));
        });
        tracing::info!("order_item_step: {:?}", order_item_step);

        let mut order_items_steps = OrderItemSteps::new();
        order_ids.iter().for_each(|order_id| {
            let mut order_item_progress_stats = HashMap::new();
            order_item_step.iter().for_each(|(order_item_id, step)| {
                let order_id_ = order_item_id_to_order_id.get(order_item_id).unwrap_or(&0);
                if order_id_ == order_id {
                    let count = order_item_progress_stats.entry(*step).or_insert(0);
                    *count += 1;
                }
            });

            order_items_steps.insert(*order_id, order_item_progress_stats);
        });

        Ok(order_items_steps)
    }
}
