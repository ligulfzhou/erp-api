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
        .with_state(state)
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateOrderParam {
    customer_id: i32,
    order_no: String,
    order_date: i32,
    delivery_date: i32,
}

async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrderParam>,
) -> ERPResult<APIEmptyResponse> {
    // check order_no exists.
    if let Ok(existing) = sqlx::query_as!(
        OrderModel,
        "select * from orders where order_no = $1",
        payload.order_no
    )
    .fetch_one(&state.db)
    .await
    {
        return Err(ERPError::AlreadyExists(format!(
            "Order with order_no: {}",
            payload.order_no
        )));
    }

    // insert into table
    sqlx::query(
        r#"
            insert into orders (customer_id, order_no, order_date, delivery_date)
            values ($1, $2, $3, $4);
        "#,
    )
    .bind(payload.customer_id)
    .bind(payload.order_no)
    .bind(payload.order_date)
    .bind(payload.delivery_date)
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
    let page_size = list_param.page_size.unwrap_or(50);
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
    let page_size = order_items_query.page_size.unwrap_or(50);
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
    Json(updateOrderParam): Json<UpdateOrderParam>,
) -> ERPResult<APIEmptyResponse> {
    let order = sqlx::query_as!(
        OrderModel,
        "select * from orders where id = $1",
        updateOrderParam.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| ERPError::NotFound(format!("Order#{} {err}", updateOrderParam.id)))?;

    let res = sqlx::query(&updateOrderParam.to_sql())
        .execute(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIEmptyResponse::new())
}
