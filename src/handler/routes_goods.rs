use crate::handler::ListParamToSQLTrait;
use crate::model::goods::GoodsModel;
use crate::model::order::{OrderItemModel, OrderModel};
use crate::response::api_response::{APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/goods", get(get_goods).post(create_order))
        .route("/api/goods/update", get(get_order_items))
        .route("/api/skus", get(get_skus))
        .route("/api/goods/skus", post(update_order))
        .route("/api/goods/sku/update", post(update_order_item))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct ListGoodsParam {
    name: Option<String>,
    goods_no: Option<String>,
    plating: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

// impl ListParamTrait for ListGoodsParam {
impl ListParamToSQLTrait for ListGoodsParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = "select * from goods ".to_string();
        let mut where_clauses = vec![];
        if let Some(name) = &self.name {
            where_clauses.push(format!(" name='{}' ", name));
        }
        if let Some(goods_no) = &self.goods_no {
            where_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if let Some(plating) = &self.plating {
            where_clauses.push(format!(" plating='{}' ", plating));
        }
        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(50);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(" offset {} limit {};", offset, page_size));

        sql
    }

    fn to_count_sql(&self) -> String {
        let mut sql = "select count(1) from goods ".to_string();
        let mut where_clauses = vec![];
        if let Some(name) = &self.name {
            where_clauses.push(format!(" name='{}' ", name));
        }
        if let Some(goods_no) = &self.goods_no {
            where_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if let Some(plating) = &self.plating {
            where_clauses.push(format!(" plating='{}' ", plating));
        }
        if where_clauses.len() > 0 {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
            sql.push_str(";");
        }

        sql
    }
}

async fn get_goods(
    State(state): State<Arc<AppState>>,
    Query(list_goods_param): Query<ListGoodsParam>,
) -> ERPResult<APIListResponse<GoodsModel>> {
    let pagination_sql = list_goods_param.to_pagination_sql();
    let goods = sqlx::query_as::<_, GoodsModel>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    let count_sql = list_goods_param.to_count_sql();
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIListResponse::new(goods, total.0 as i32))
}

#[derive(Debug, Deserialize)]
struct ListSKUsParam {
    name: Option<String>,
    goods_no: Option<String>,
    plating: Option<String>,
    color: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl ListParamToSQLTrait for ListSKUsParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = "select * from goods ".to_string();
        let mut where_clauses = vec![];
        if let Some(name) = &self.name {
            where_clauses.push(format!(" name='{}' ", name));
        }
        if let Some(goods_no) = &self.goods_no {
            where_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if let Some(plating) = &self.plating {
            where_clauses.push(format!(" plating='{}' ", plating));
        }
        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(50);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(" offset {} limit {};", offset, page_size));

        sql
    }

    fn to_count_sql(&self) -> String {
        let mut sql = "select count(1) from goods ".to_string();
        let mut where_clauses = vec![];
        if let Some(name) = &self.name {
            where_clauses.push(format!(" name='{}' ", name));
        }
        if let Some(goods_no) = &self.goods_no {
            where_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if let Some(plating) = &self.plating {
            where_clauses.push(format!(" plating='{}' ", plating));
        }
        if where_clauses.len() > 0 {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
            sql.push_str(";");
        }

        sql
    }
}

async fn get_skus(
    State(state): State<Arc<AppState>>,
    Query(list_goods_param): Query<ListGoodsParam>,
) -> ERPResult<APIListResponse<GoodsModel>> {
    let pagination_sql = list_goods_param.to_pagination_sql();
    let goods = sqlx::query_as::<_, GoodsModel>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    let count_sql = list_goods_param.to_count_sql();
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIListResponse::new(goods, total.0 as i32))
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateGoodsParam {
    customer_id: i32,
    order_no: String,
    order_date: i32,
    delivery_date: i32,
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

/// todo: add more params
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
pub struct UpdateOrderItemParam {
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

#[cfg(test)]
mod tests {
    use crate::handler::routes_goods::ListGoodsParam;
    use crate::handler::ListParamToSQLTrait;

    #[test]
    fn test() {
        let params = ListGoodsParam {
            name: Some("name".to_string()),
            goods_no: Some("goods_no".to_string()),
            plating: None,
            page: None,
            page_size: None,
        };
        let sql = params.to_pagination_sql();
        let count_sql = params.to_count_sql();
        println!("{}", params.to_pagination_sql());
        println!("{}", count_sql);
    }
}
