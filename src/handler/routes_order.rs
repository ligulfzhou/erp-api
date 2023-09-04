use crate::constants::DEFAULT_PAGE_SIZE;
use crate::dto::dto_orders::{
    OrderDto, OrderGoodsDto, OrderGoodsItemDto, OrderGoodsWithItemDto, OrderWithStepsDto,
};
use crate::handler::ListParamToSQLTrait;
use crate::model::order::OrderModel;
use crate::model::progress::ProgressModel;
use crate::response::api_response::{APIDataResponse, APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::WithRejection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/orders", get(get_orders).post(create_order))
        .route("/api/order/detail", get(order_detail))
        .route("/api/order/update", post(update_order))
        .route("/api/order/items", get(get_order_items))
        .route("/api/order/goods/update", post(update_order_goods))
        .route("/api/order/item/update", post(update_order_item))
        .route("/api/order/item/delete", post(delete_order_item))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct DeleteOrderItem {
    id: i32,
}

impl DeleteOrderItem {
    fn to_sql(&self) -> String {
        format!("delete from order_items where id = {}", self.id)
    }
}

async fn delete_order_item(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<DeleteOrderItem>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    state.execute_sql(&param.to_sql()).await?;
    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateOrderParam {
    customer_no: String,
    order_no: String,
    order_date: String,
    delivery_date: Option<String>,
    is_urgent: bool,
    is_return_order: bool,
}

async fn create_order(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateOrderParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    // 检查订单号是否已存在.
    if sqlx::query_as!(
        OrderModel,
        "select * from orders where order_no = $1",
        payload.order_no
    )
    .fetch_one(&state.db)
    .await
    .is_ok()
    {
        return Err(ERPError::AlreadyExists(format!(
            "订单编号{} 已存在",
            payload.order_no
        )));
    }

    // 获取客户的ID
    let (customer_id,) = sqlx::query_as::<_, (i32,)>(&format!(
        "select id from customers where customer_no = '{}'",
        payload.customer_no
    ))
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    let delivery_date = {
        if payload.delivery_date.is_none()
            || payload.delivery_date.as_deref().unwrap_or("").is_empty()
        {
            "null".to_string()
        } else {
            format!("'{}'", payload.delivery_date.as_deref().unwrap())
        }
    };

    // 插入订单
    let sql = format!(
        r#"insert into orders (customer_id, order_no, order_date, delivery_date, is_urgent, is_return_order)
               values ('{}', '{}', '{}', {}, {}, {});"#,
        customer_id,
        payload.order_no,
        payload.order_date,
        delivery_date,
        payload.is_urgent,
        payload.is_return_order
    );

    state.execute_sql(&sql).await?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct DetailParam {
    id: Option<i32>,
    order_no: Option<String>,
}

async fn order_detail(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<DetailParam>, ERPError>,
) -> ERPResult<APIDataResponse<OrderDto>> {
    let mut sql = r#"
    select 
        o.id, o.customer_id, o.order_no, o.order_date, o.delivery_date, o.is_return_order, o.is_urgent,
        c.name as customer_name, c.phone as customer_phone, c.address as customer_address, c.customer_no
    from orders o, customers c
    where o.customer_id = c.id
    "#.to_string();

    let id = param.id.unwrap_or(0);
    if id > 0 {
        sql.push_str(&format!(" and o.id={id}"))
    }
    let order_no = param.order_no.as_deref().unwrap_or("");
    if !order_no.is_empty() {
        sql.push_str(&format!(" and o.order_no='{order_no}'"));
    }

    let order_dto = sqlx::query_as::<_, OrderDto>(&sql)
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIDataResponse::new(order_dto))
}

#[derive(Debug, Deserialize)]
struct ListParam {
    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,

    customer_no: Option<String>,
    order_no: Option<String>,
    sort_col: Option<String>,
    sort: Option<String>, // desc/asc: default desc
}

impl ListParamToSQLTrait for ListParam {
    fn to_pagination_sql(&self) -> String {
        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let offset = (page - 1) * page_size;

        let mut sql = r#"
        select 
            o.id, o.order_no, o.order_date, o.delivery_date, o.is_return_order, o.is_urgent, o.customer_id,
            c.customer_no, c.address as customer_address, c.name as customer_name, c.phone as customer_phone 
        from orders o, customers c
        where o.customer_id = c.id
        "#.to_string();
        let customer_no = self.customer_no.as_deref().unwrap_or("");
        if !customer_no.is_empty() {
            sql.push_str(&format!(" and c.customer_no = '{}'", customer_no));
        }
        let order_no = self.order_no.as_deref().unwrap_or("");
        if !order_no.is_empty() {
            sql.push_str(&format!(" and o.order_no like '%{}%'", order_no));
        }

        sql.push_str(&format!(
            " order by o.id desc offset {} limit {};",
            offset, page_size
        ));

        tracing::info!("get orders sql: {:?}", sql);
        sql
    }

    fn to_count_sql(&self) -> String {
        let customer_no = self.customer_no.as_deref().unwrap_or("");
        let order_no = self.order_no.as_deref().unwrap_or("");
        if customer_no.is_empty() && order_no.is_empty() {
            "select count(1) from orders;".to_string()
        } else if customer_no.is_empty() && !order_no.is_empty() {
            format!(
                "select count(1) from orders where order_no like '%{}%';",
                order_no
            )
        } else if !customer_no.is_empty() && order_no.is_empty() {
            format!(
                r#"
                select count(1) 
                from orders o, customers c
                where o.customer_id = c.id
                and c.customer_no = '{}'
                "#,
                customer_no
            )
        } else {
            format!(
                r#"
                select count(1) 
                from orders o, customers c
                where o.customer_id = c.id 
                and c.customer_no = '{}' and o.order_no like '%{}%'
                "#,
                customer_no, order_no
            )
        }
    }
}

async fn get_orders(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<ListParam>, ERPError>,
) -> ERPResult<APIListResponse<OrderWithStepsDto>> {
    tracing::info!("get_orders: ....");

    let order_dtos = sqlx::query_as::<_, OrderDto>(&param.to_pagination_sql())
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    if order_dtos.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }

    // 去获取各产品的流程
    let order_ids = order_dtos
        .iter()
        .map(|order| order.id)
        .collect::<Vec<i32>>();

    let order_items_steps = ProgressModel::get_progress_status(&state.db, order_ids).await?;
    println!("{:#?}", order_items_steps);

    let order_with_step_dtos = order_dtos
        .into_iter()
        .map(|order_dto| {
            let steps = order_items_steps.get(&order_dto.id).unwrap();
            OrderWithStepsDto::from_order_dto_and_steps(order_dto, steps.clone())
        })
        .collect::<Vec<OrderWithStepsDto>>();

    let count: (i64,) = sqlx::query_as(&param.to_count_sql())
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(order_with_step_dtos, count.0 as i32))
}

#[derive(Debug, Deserialize)]
struct OrderItemsQuery {
    order_id: i32,
    page: Option<i32>,

    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl ListParamToSQLTrait for OrderItemsQuery {
    fn to_pagination_sql(&self) -> String {
        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let offset = (page - 1) * page_size;
        format!(
            r#"
            select 
                og.id, og.order_id, og.goods_id, g.goods_no, g.name, g.image, 
                g.plating, og.package_card, og.package_card_des
            from order_goods og, goods g
            where og.goods_id = g.id and og.order_id = {}
            order by og.id offset {} limit {}
            "#,
            self.order_id, offset, page_size
        )
    }

    fn to_count_sql(&self) -> String {
        format!(
            "select count(1) from order_goods where order_id = {}",
            self.order_id
        )
    }
}

async fn get_order_items(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<OrderItemsQuery>, ERPError>,
) -> ERPResult<APIListResponse<OrderGoodsWithItemDto>> {
    if param.order_id == 0 {
        return Err(ERPError::ParamNeeded(param.order_id.to_string()));
    }

    // 获取order_good
    let order_goods = sqlx::query_as::<_, OrderGoodsDto>(&param.to_pagination_sql())
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;
    if order_goods.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }
    println!("order_goods: {:?}, len: {}", order_goods, order_goods.len());

    // 从order_goods里拿出goods_ids
    let goods_ids_str = order_goods
        .iter()
        .map(|goods| goods.goods_id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    // 用goods_ids去获取order_items
    let order_items_dto = sqlx::query_as::<_, OrderGoodsItemDto>(&format!(
        r#"
        select 
            oi.id, oi.order_id, oi.goods_id, oi.sku_id, s.color,
            s.sku_no, oi.count, oi.unit, oi.unit_price, oi.total_price, oi.notes
        from order_items oi, skus s
        where oi.sku_id = s.id
            and oi.order_id = {} and oi.goods_id in ({})
        order by id;
        "#,
        param.order_id, goods_ids_str
    ))
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    println!(
        "order_items: {:?}, len: {}",
        order_items_dto,
        order_items_dto.len()
    );
    let mut gid_order_item_dtos = HashMap::new();
    for item in order_items_dto.into_iter() {
        let dtos = gid_order_item_dtos.entry(item.goods_id).or_insert(vec![]);
        dtos.push(item);
    }
    let empty_array: Vec<OrderGoodsItemDto> = vec![];
    let order_goods_dtos = order_goods
        .into_iter()
        .map(|order_good| {
            let items = gid_order_item_dtos
                .get(&order_good.goods_id)
                .unwrap_or(&empty_array);
            OrderGoodsWithItemDto::from_order_with_goods(order_good, items.clone())
        })
        .collect::<Vec<OrderGoodsWithItemDto>>();

    let count: (i64,) = sqlx::query_as(&param.to_count_sql())
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(order_goods_dtos, count.0 as i32))
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateOrderParam {
    id: i32,
    order_no: String,
    customer_id: i32,
    order_date: String,
    delivery_date: Option<String>,
    is_return_order: bool,
    is_urgent: bool,
}

impl UpdateOrderParam {
    pub fn to_sql(&self) -> String {
        let delivery_date = {
            if self.delivery_date.is_none()
                || self.delivery_date.as_deref().unwrap_or("").is_empty()
            {
                "null".to_string()
            } else {
                format!("'{}'", self.delivery_date.as_deref().unwrap())
            }
        };

        format!(
            "update orders set order_no='{}', customer_id={}, order_date='{}', delivery_date={}, is_return_order={}, is_urgent={} where id={};", 
            self.order_no, self.customer_id, self.order_date, delivery_date, self.is_return_order, self.is_urgent, self.id)
    }
}

async fn update_order(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateOrderParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let _order = sqlx::query_as!(OrderModel, "select * from orders where id = $1", payload.id)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::NotFound(format!("Order#{} {err}", payload.id)))?;

    tracing::info!("update order sql: {}", payload.to_sql());
    state.execute_sql(&payload.to_sql()).await?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct UpdateOrderItemParam {
    id: Option<i32>,
    order_id: Option<i32>,
    goods_id: Option<i32>,
    sku_id: Option<i32>,
    count: i32,
    unit: Option<String>,
    unit_price: Option<i32>,
    total_price: Option<i32>,
}

impl UpdateOrderItemParam {
    fn to_insert_sql(&self) -> String {
        let mut kv_pairs: Vec<(_, _)> = vec![];
        kv_pairs.push(("order_id", self.order_id.unwrap_or(0).to_string()));
        kv_pairs.push(("sku_id", self.sku_id.unwrap_or(0).to_string()));
        kv_pairs.push(("goods_id", self.goods_id.unwrap_or(0).to_string()));
        kv_pairs.push(("count", self.count.to_string()));
        kv_pairs.push(("unit", format!("'{}'", self.unit.as_deref().unwrap_or(""))));
        if let Some(unit_price) = self.unit_price {
            kv_pairs.push(("unit_price", unit_price.to_string()))
        }
        if let Some(total_price) = self.total_price {
            kv_pairs.push(("total_price", total_price.to_string()))
        }

        let keys = kv_pairs
            .iter()
            .map(|kv| kv.0)
            .collect::<Vec<&str>>()
            .join(",");
        tracing::debug!("keys: {:?}", keys);

        let values = kv_pairs
            .iter()
            .map(|kv| kv.1.as_str())
            .collect::<Vec<&str>>()
            .join(",");
        tracing::debug!("values: {:?}", values);

        let sql = format!("insert into order_items ({}) values ({})", keys, values);
        tracing::debug!("sql: {:?}", sql);

        sql
    }

    fn to_update_sql(&self) -> String {
        let mut where_clauses = vec![];
        where_clauses.push(format!("count={}", self.count));
        if let Some(unit) = &self.unit {
            where_clauses.push(format!("unit='{}'", unit));
        }
        if let Some(unit_price) = &self.unit_price {
            where_clauses.push(format!("unit_price={}", unit_price));
        }
        if let Some(total_price) = &self.total_price {
            where_clauses.push(format!("total_price={}", total_price));
        }

        let sql = format!(
            "update order_items set {} where id={}",
            where_clauses.join(","),
            self.id.unwrap()
        );

        sql
    }
}

async fn update_order_item(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateOrderItemParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    if let Some(id) = payload.id {
        // 修改数据
        tracing::info!(
            "=> handler update_order_item: update sql: {:?}",
            payload.to_update_sql()
        );
        state.execute_sql(&payload.to_update_sql()).await?;
    } else {
        // 新增
        let order_id = payload.order_id.expect("订单ID");
        let sku_id = payload.sku_id.expect("sku ID");

        // check if sku_id exists
        let ids_with_this_sku_id = sqlx::query_as::<_, (i32,)>(&format!(
            "select id from order_items where order_id={} and sku_id={}",
            order_id, sku_id
        ))
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?
        .iter()
        .map(|id| id.0)
        .collect::<Vec<i32>>();
        tracing::info!("ids_with_this_sku_id: {:?}", ids_with_this_sku_id);

        if !ids_with_this_sku_id.is_empty() {
            return Err(ERPError::AlreadyExists("该商品已添加".to_string()));
        }

        // insert
        tracing::info!(
            "=> handler update_order_item: insert sql: {:?}",
            payload.to_insert_sql()
        );

        state.execute_sql(&payload.to_insert_sql()).await?;
    }

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct UpdateOrderGoodsParam {
    id: Option<i32>,
    order_id: Option<i32>,
    goods_id: Option<i32>,
    sku_id: Option<i32>,
    count: i32,
    unit: Option<String>,
    unit_price: Option<i32>,
    total_price: Option<i32>,
}

impl UpdateOrderGoodsParam {
    fn to_insert_sql(&self) -> String {
        let mut kv_pairs: Vec<(_, _)> = vec![];
        kv_pairs.push(("order_id", self.order_id.unwrap_or(0).to_string()));
        kv_pairs.push(("sku_id", self.sku_id.unwrap_or(0).to_string()));
        kv_pairs.push(("goods_id", self.goods_id.unwrap_or(0).to_string()));
        kv_pairs.push(("count", self.count.to_string()));
        kv_pairs.push(("unit", format!("'{}'", self.unit.as_deref().unwrap_or(""))));
        if let Some(unit_price) = self.unit_price {
            kv_pairs.push(("unit_price", unit_price.to_string()))
        }
        if let Some(total_price) = self.total_price {
            kv_pairs.push(("total_price", total_price.to_string()))
        }

        let keys = kv_pairs
            .iter()
            .map(|kv| kv.0)
            .collect::<Vec<&str>>()
            .join(",");
        tracing::debug!("keys: {:?}", keys);

        let values = kv_pairs
            .iter()
            .map(|kv| kv.1.as_str())
            .collect::<Vec<&str>>()
            .join(",");
        tracing::debug!("values: {:?}", values);

        let sql = format!("insert into order_items ({}) values ({})", keys, values);
        tracing::debug!("sql: {:?}", sql);

        sql
    }

    fn to_update_sql(&self) -> String {
        let mut where_clauses = vec![];
        where_clauses.push(format!("count={}", self.count));
        if let Some(unit) = &self.unit {
            where_clauses.push(format!("unit='{}'", unit));
        }
        if let Some(unit_price) = &self.unit_price {
            where_clauses.push(format!("unit_price={}", unit_price));
        }
        if let Some(total_price) = &self.total_price {
            where_clauses.push(format!("total_price={}", total_price));
        }

        let sql = format!(
            "update order_items set {} where id={}",
            where_clauses.join(","),
            self.id.unwrap()
        );

        sql
    }
}

async fn update_order_goods(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateOrderGoodsParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    if let Some(id) = payload.id {
        // 修改数据
        tracing::info!(
            "=> handler update_order_item: update sql: {:?}",
            payload.to_update_sql()
        );
        state.execute_sql(&payload.to_update_sql()).await?;
    } else {
        // 新增
        let order_id = payload.order_id.expect("订单ID");
        let sku_id = payload.sku_id.expect("sku ID");

        // check if sku_id exists
        let ids_with_this_sku_id = sqlx::query_as::<_, (i32,)>(&format!(
            "select id from order_items where order_id={} and sku_id={}",
            order_id, sku_id
        ))
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?
        .iter()
        .map(|id| id.0)
        .collect::<Vec<i32>>();
        tracing::info!("ids_with_this_sku_id: {:?}", ids_with_this_sku_id);

        if !ids_with_this_sku_id.is_empty() {
            return Err(ERPError::AlreadyExists("该商品已添加".to_string()));
        }

        // insert
        tracing::info!(
            "=> handler update_order_item: insert sql: {:?}",
            payload.to_insert_sql()
        );

        state.execute_sql(&payload.to_insert_sql()).await?;
    }

    Ok(APIEmptyResponse::new())
}

#[cfg(test)]
mod tests {
    use crate::handler::routes_order::CreateOrderParam;
    use anyhow::Result;
    use tokio;

    #[tokio::test]
    async fn test_create_order() -> Result<()> {
        let param = CreateOrderParam {
            customer_no: "L1007".to_string(),
            order_no: "order_no_101".to_string(),
            order_date: "2022-03-09".to_string(),
            delivery_date: None,
            is_urgent: false,
            is_return_order: false,
        };
        let client = httpc_test::new_client("http://localhost:9100")?;

        client
            .do_post("/api/orders", serde_json::json!(param))
            .await?
            .print()
            .await?;

        Ok(())
    }
}
