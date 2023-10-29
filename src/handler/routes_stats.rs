use crate::constants::DEFAULT_PAGE_SIZE;
use crate::dto::dto_goods::{GoodsDto, SKUModelDto};
use crate::dto::dto_stats::{ReturnOrderGoodsStat, ReturnOrderItemStat, ReturnOrderStat};
use crate::response::api_response::APIListResponse;
use crate::service::goods_service::GoodsService;
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
        .route("/api/stats/produce", get(order_stats))
        .route(
            "/api/stats/return/orders/by/goods",
            get(list_return_orders_by_goods),
        )
        .route(
            "/api/stats/return/orders/by/items",
            get(list_return_orders_by_items),
        )
        .with_state(state)
}

#[derive(Deserialize)]
pub struct OrderStatParam {
    customer_no: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

async fn order_stats(
    State(state): State<Arc<AppState>>,
) -> ERPResult<APIListResponse<ReturnOrderStat>> {
    todo!()
}

#[derive(Deserialize)]
pub struct ReturnOrderStatParam {
    customer_no: Option<String>,
    sorter_field: Option<String>,
    sorter_order: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

async fn list_return_orders_by_goods(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(params), _): WithRejection<Query<ReturnOrderStatParam>, ERPError>,
) -> ERPResult<APIListResponse<ReturnOrderGoodsStat>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    let goods_id_with_count_and_sum = sqlx::query!(
        r#"
        select
            og.goods_id,
            count(distinct(o.order_no)) as order_count,
            count(1) as item_count,
            sum(oi.count)
        from order_goods og, order_items oi, orders o
        where oi.order_goods_id=og.id and og.order_id = o.id
        group by og.goods_id
        having count(distinct(o.order_no)) > 1
        order by count(1) desc, sum(oi.count) desc
        offset $1 limit $2
        "#,
        offset as i64,
        page_size as i64
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    print!("{:?}", goods_id_with_count_and_sum);

    let goods_ids = goods_id_with_count_and_sum
        .iter()
        .map(|r| r.goods_id)
        .collect::<Vec<i32>>();

    if goods_ids.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }

    let goods_id_to_goods_model = GoodsService::get_goods_dtos(&state.db, &goods_ids)
        .await?
        .into_iter()
        .map(|item| (item.id, item))
        .collect::<HashMap<i32, GoodsDto>>();

    let goods_id_to_count_and_sum = goods_id_with_count_and_sum
        .into_iter()
        .map(|r| {
            (
                r.goods_id,
                (r.order_count.unwrap_or(0) as i32, r.sum.unwrap_or(0) as i32),
            )
        })
        .collect::<HashMap<i32, (i32, i32)>>();

    let sku_id_to_cnt_and_sum = sqlx::query!(
        r#"
        select
            oi.sku_id, count(1), sum(oi.count)
        from order_items oi, order_goods og
        where
            oi.order_goods_id = og.id and og.goods_id = any($1)
        group by oi.sku_id
        -- having count(1) > 0
        order by count(1) desc, sum(oi.count) desc
        "#,
        &goods_ids
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .into_iter()
    .map(|record| {
        (
            record.sku_id,
            (
                record.count.unwrap_or(0) as i32,
                record.sum.unwrap_or(0) as i32,
            ),
        )
    })
    .collect::<HashMap<i32, (i32, i32)>>();
    tracing::info!("sku_id_to_cnt_and_sum: {:?}", sku_id_to_cnt_and_sum);

    let skus = GoodsService::get_sku_dtos_with_goods_ids(&state.db, &goods_ids).await?;
    // let skus = sqlx::query_as!(
    //     SKUModelWithoutImageAndPackageDto,
    //     r#"
    //     select
    //         s.id, s.sku_no, g.customer_no, g.name, g.goods_no, g.id as goods_id,
    //         g.plating, s.color, s.color2, s.notes
    //     from skus s, goods g
    //     where s.goods_id = g.id
    //         and s.goods_id = any($1)
    //     "#,
    //     &goods_ids
    // )
    // .fetch_all(&state.db)
    // .await
    // .map_err(ERPError::DBError)?;

    // let sku_id_to_goods_id = skus
    //     .iter()
    //     .map(|sku| (sku.id, sku.goods_id))
    //     .collect::<HashMap<i32, i32>>();

    let mut goods_id_to_vec_sku_ids: HashMap<i32, Vec<i32>> = HashMap::new();
    skus.iter().for_each(|sku| {
        goods_id_to_vec_sku_ids
            .entry(sku.goods_id)
            .or_insert(vec![])
            .push(sku.id)
    });

    let id_to_skus = skus
        .into_iter()
        .map(|sku| (sku.id, sku))
        .collect::<HashMap<i32, SKUModelDto>>();

    let mut stats: Vec<ReturnOrderGoodsStat> = vec![];

    let empty_count_and_sum = (0, 0);
    let empty_sku_ids: Vec<i32> = vec![];
    goods_ids.iter().for_each(|goods_id| {
        let count_sum = goods_id_to_count_and_sum
            .get(goods_id)
            .unwrap_or(&empty_count_and_sum);

        let goods_model = goods_id_to_goods_model.get(goods_id).unwrap();

        let skus_stats = goods_id_to_vec_sku_ids
            .get(goods_id)
            .unwrap_or(&empty_sku_ids)
            .iter()
            .map(|sku_id| {
                let sku = id_to_skus.get(sku_id).unwrap();
                let count_and_sum = sku_id_to_cnt_and_sum
                    .get(sku_id)
                    .unwrap_or(&empty_count_and_sum);
                ReturnOrderItemStat {
                    sku: sku.clone(),
                    count: count_and_sum.0,
                    sum: count_and_sum.1,
                }
            })
            .collect::<Vec<ReturnOrderItemStat>>();

        let stat = ReturnOrderGoodsStat {
            goods: goods_model.clone(),
            skus: skus_stats,
            count: count_sum.0,
            sum: count_sum.1,
        };

        stats.push(stat);
    });

    let count = sqlx::query!(
        "select goods_id, count(1) from order_goods group by goods_id having count(1) > 1;"
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .len() as i32;

    Ok(APIListResponse::new(stats, count))
}

async fn list_return_orders_by_items(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(params), _): WithRejection<Query<ReturnOrderStatParam>, ERPError>,
) -> ERPResult<APIListResponse<ReturnOrderItemStat>> {
    let page = params.page.unwrap_or(1);
    let page_size = params.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    let sku_id_and_cnt = sqlx::query!(
        r#"
        select sku_id, count(1), sum(count) from order_items
        group by sku_id
        having count(1) > 1
        order by count(1) desc, sum(count) desc, sku_id desc
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

    let id_to_skus = GoodsService::get_sku_dtos(&state.db, &sku_ids)
        .await?
        .into_iter()
        .map(|item| (item.id, item))
        .collect::<HashMap<i32, SKUModelDto>>();

    // let id_to_skus = sqlx::query_as!(
    //     SKUModelDto,
    //     r#"
    //     select
    //         s.id, s.sku_no, g.customer_no, g.name, g.goods_no, g.id as goods_id,
    //         g.images, g.plating, s.color, s.color2, s.notes, g.package_card
    //     from skus s, goods g
    //     where s.goods_id = g.id
    //         and s.id = any($1)
    //     "#,
    //     &sku_ids
    // )
    // .fetch_all(&state.db)
    // .await
    // .map_err(ERPError::DBError)?
    // .into_iter()
    // .map(|sku| (sku.id, sku))
    // .collect::<HashMap<i32, SKUModelDto>>();

    let empty_count_and_sum = (0, 0);

    let sku_stats = sku_ids
        .iter()
        .filter_map(|sku_id| {
            let count_and_sum = sku_id_to_count_and_sum
                .get(sku_id)
                .unwrap_or(&empty_count_and_sum);

            let sku = id_to_skus.get(sku_id);
            sku.map(|sku_dto| ReturnOrderItemStat {
                sku: sku_dto.clone(),
                count: count_and_sum.0,
                sum: count_and_sum.1,
            })
        })
        .collect::<Vec<ReturnOrderItemStat>>();

    let count = sqlx::query!(
        r#"
        select sku_id, count(1) from order_items
        group by sku_id
        having count(1) > 1;
        "#
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .len() as i32;

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
