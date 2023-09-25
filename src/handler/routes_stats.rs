use crate::constants::DEFAULT_PAGE_SIZE;
use crate::dto::dto_goods::SKUModelDto;
use crate::dto::dto_stats::ReturnOrderStat;
use crate::response::api_response::APIListResponse;
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::get;
use axum::Router;
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/stats/orders", get(order_stats))
        .route("/api/stats/return/orders", get(list_return_orders))
        .with_state(state)
}

async fn order_stats(
    State(state): State<Arc<AppState>>,
) -> ERPResult<APIListResponse<ReturnOrderStat>> {
    todo!()
}

#[derive(Deserialize)]
pub struct ReturnOrderParam {
    customer_no: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

async fn list_return_orders(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(params), _): WithRejection<Query<ReturnOrderParam>, ERPError>,
) -> ERPResult<APIListResponse<ReturnOrderStat>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    let sku_id_and_cnt = sqlx::query!(
        r#"
        select sku_id, count(1), sum(count) from order_items
        group by sku_id
        having count(1) > 1
        order by count(1) desc, sku_id desc
        offset $1 limit $2
        "#,
        offset as i64,
        page_size as i64
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    let sku_ids = sku_id_and_cnt
        .iter()
        .map(|r| r.sku_id)
        .collect::<Vec<i32>>();

    if sku_ids.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }

    let sku_id_to_count_and_sum = sku_id_and_cnt
        .into_iter()
        .map(|r| {
            (
                r.sku_id,
                (r.count.unwrap_or(0) as i32, r.sum.unwrap_or(0) as i32),
            )
        })
        .collect::<HashMap<i32, (i32, i32)>>();

    let id_to_skus = sqlx::query_as!(
        SKUModelDto,
        r#"
        select
            s.id, s.sku_no, g.customer_no, g.name, g.goods_no, g.id as goods_id,
            g.image, g.plating, s.color, s.color2, s.notes, g.package_card
        from skus s, goods g
        where s.goods_id = g.id
            and s.id = any($1)
        "#,
        &sku_ids
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .into_iter()
    .map(|sku| (sku.id, sku))
    .collect::<HashMap<i32, SKUModelDto>>();

    let empty_count_and_sum = (0, 0);

    let sku_stats = sku_ids
        .iter()
        .filter_map(|sku_id| {
            let count_and_sum = sku_id_to_count_and_sum
                .get(sku_id)
                .unwrap_or(&empty_count_and_sum);

            let sku = id_to_skus.get(sku_id);
            sku.map(|sku_dto| ReturnOrderStat {
                sku: sku_dto.clone(),
                count: count_and_sum.0,
                sum: count_and_sum.1,
            })
        })
        .collect::<Vec<ReturnOrderStat>>();

    let count = sqlx::query!(
        r#"
        select count(1) from order_items
        group by sku_id
        having count(1) > 1;
        "#
    )
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .count
    .unwrap_or(0) as i32;

    Ok(APIListResponse::new(sku_stats, count))
}

#[cfg(test)]
mod tests {
    use crate::handler::routes_login::LoginPayload;

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

        client.do_get("/api/account/info").await?.print().await?;
        Ok(())
    }
}
