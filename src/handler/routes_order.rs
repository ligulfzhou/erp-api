use crate::constants::DEFAULT_PAGE_SIZE;
use crate::model::order::{OrderItemModel, OrderModel};
use crate::response::api_response::{APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/orders", get(get_orders).post(create_order))
        .route("/api/order/update", post(update_order))
        .route("/api/order/items", get(get_order_items))
        .route("/api/order/item/update", post(update_order_item))
        .route("/api/order/item/materials", get())
        .with_state(state)
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateOrderParam {
    customer_id: i32,
    order_no: String,
    order_date: i32,
    delivery_date: i32,
}

impl CreateOrderParam {
    fn to_sql(&self) -> String {
        format!(
            "insert into orders (customer_id, order_no, order_date, delivery_date)
            values ('{}', '{}', {}, {});",
            self.customer_id, self.order_no, self.order_date, self.delivery_date
        )
    }
}

async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrderParam>,
) -> ERPResult<APIEmptyResponse> {
    // check order_no exists.
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
            "Order with order_no: {}",
            payload.order_no
        )));
    }

    // insert into table
    sqlx::query(&payload.to_sql())
        .execute(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct ListParam {
    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
    sort_col: Option<String>,
    sort: Option<String>, // desc/asc: default desc
}

async fn get_orders(
    State(state): State<Arc<AppState>>,
    Query(list_param): Query<ListParam>,
) -> ERPResult<APIListResponse<OrderModel>> {
    let page = list_param.page.unwrap_or(1);
    let page_size = list_param.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    let orders = sqlx::query_as!(
        OrderModel,
        "select * from orders offset $1 limit $2",
        offset as i64,
        page_size as i64
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| ERPError::DBError(err))?;

    let count = sqlx::query!("select count(1) from orders")
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?
        .count
        .unwrap_or(0);

    Ok(APIListResponse::new(orders, count as i32))
}

#[derive(Debug, Deserialize)]
struct OrderItemsQuery {
    order_id: i32,
    page: Option<i32>,

    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl OrderItemsQuery {
    pub(crate) fn to_paginate_sql(&self) -> String {
        todo!()
    }

    pub(crate) fn to_count_sql(&self) -> String {
        todo!()
    }
}

async fn get_order_items(
    State(state): State<Arc<AppState>>,
    Query(order_items_query): Query<OrderItemsQuery>,
) -> ERPResult<APIListResponse<OrderItemModel>> {
    if order_items_query.order_id == 0 {
        return Err(ERPError::ParamNeeded(
            order_items_query.order_id.to_string(),
        ));
    }

    let page = order_items_query.page.unwrap_or(1);
    let page_size = order_items_query.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    let order_items = sqlx::query_as!(
        OrderItemModel,
        "select * from order_items where order_id = $1 order by id desc offset $2 limit $3",
        order_items_query.order_id,
        offset as i64,
        page_size as i64
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| ERPError::DBError(err))?;

    let count = sqlx::query!(
        "select count(1) from order_items where order_id = $1",
        order_items_query.order_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| ERPError::DBError(err))?
    .count
    .unwrap_or(0);

    Ok(APIListResponse::new(order_items, count as i32))
}

#[derive(Debug, Deserialize)]
struct UpdateOrderParam {
    id: i32,
    order_no: String,
    customer_id: i32,
    order_date: i32,
    delivery_date: i32,
}

impl UpdateOrderParam {
    pub fn to_sql(&self) -> String {
        format!("update orders set order_no = '{}', customer_id = {}, order_date={}, delivery_date={} where id={}", self.order_no, self.customer_id, self.order_date, self.delivery_date, self.id)
    }
}

async fn update_order(
    State(state): State<Arc<AppState>>,
    Json(update_order_param): Json<UpdateOrderParam>,
) -> ERPResult<APIEmptyResponse> {
    let order = sqlx::query_as!(
        OrderModel,
        "select * from orders where id = $1",
        update_order_param.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| ERPError::NotFound(format!("Order#{} {err}", update_order_param.id)))?;

    let _ = sqlx::query(&update_order_param.to_sql())
        .execute(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct UpdateOrderItemParam {
    id: i32,
    order_id: i32,
    sku_id: i32,
    package_card: Option<String>,
    package_card_des: Option<String>,
    count: i32,
    unit: Option<String>,
    unit_price: Option<i32>,
    total_price: Option<i32>,
    notes: Option<String>,
}

impl UpdateOrderItemParam {
    fn to_sql(&self) -> String {
        let mut sql = format!(
            "update order_items set order_id = {}, sku_id = {}, count={}",
            self.order_id, self.sku_id, self.count
        );
        if let Some(package_card) = &self.package_card {
            sql.push_str(&format!(", package_card = '{}'", package_card));
        }
        if let Some(package_card_des) = &self.package_card_des {
            sql.push_str(&format!(", package_card_des = '{}'", package_card_des));
        }
        if let Some(unit) = &self.unit {
            sql.push_str(&format!(", unit = '{}'", unit));
        }
        if let Some(unit_price) = &self.unit_price {
            sql.push_str(&format!(", unit_price = {}", unit_price));
        }
        if let Some(total_price) = &self.total_price {
            sql.push_str(&format!(", total_price = '{}'", total_price));
        }
        if let Some(notes) = &self.notes {
            sql.push_str(&format!(", notes = {}", notes));
        }

        sql
    }
}

async fn update_order_item(
    State(state): State<Arc<AppState>>,
    Json(update_order_item_param): Json<UpdateOrderItemParam>,
) -> ERPResult<APIEmptyResponse> {
    let _ = sqlx::query_as!(
        OrderItemModel,
        "select * from order_items where id = $1",
        update_order_item_param.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| ERPError::NotFound(format!("OrderItem#{} {err}", update_order_item_param.id)))?;

    sqlx::query(&update_order_item_param.to_sql())
        .execute(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct OrderItemMaterialParam {
    pub order_id: i32,
    pub order_item_id: i32,
    pub name: Option<String>,
    pub color: Option<String>,
    // material_id   integer, -- 材料ID  (暂时先不用)
    pub single: Option<i32>,   //  integer, -- 单数      ？小数
    pub count: Option<i32>,    //  integer, -- 数量      ？小数
    pub total: Option<i32>,    //  integer, -- 总数(米)  ? 小数
    pub stock: Option<i32>,    //  integer, -- 库存 ?
    pub debt: Option<i32>,     //  integer, -- 欠数
    pub notes: Option<String>, //     text     -- 备注
}
