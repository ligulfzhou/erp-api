use crate::constants::DEFAULT_PAGE_SIZE;
use crate::dto::dto_orders::OrderDto;
use crate::handler::ListParamToSQLTrait;
use crate::model::customer::CustomerModel;
use crate::model::order::{OrderItemMaterialModel, OrderItemModel, OrderModel};
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
        .route("/api/order/item/update", post(update_order_item))
        .route(
            "/api/order/item/materials",
            get(get_order_item_materials).post(add_order_item_materials),
        )
        .route(
            "/api/order/item/material/update",
            post(update_order_item_material),
        )
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
    WithRejection(Json(payload), _): WithRejection<Json<CreateOrderParam>, ERPError>,
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
        .map_err(ERPError::DBError)?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct DetailParam {
    id: i32,
}

async fn order_detail(
    State(state): State<Arc<AppState>>,
    Query(param): Query<DetailParam>,
) -> ERPResult<APIDataResponse<OrderDto>> {
    let order =
        sqlx::query_as::<_, OrderModel>(&format!("select * from orders where id={}", param.id))
            .fetch_one(&state.db)
            .await
            .map_err(ERPError::DBError)?;

    let customer = sqlx::query_as::<_, CustomerModel>(&format!(
        "select * from customers where id={}",
        order.customer_id
    ))
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    Ok(APIDataResponse::new(OrderDto::from(order, customer)))
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
    Query(param): Query<ListParam>,
) -> ERPResult<APIListResponse<OrderDto>> {
    let page = param.page.unwrap_or(1);
    let page_size = param.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    let orders = sqlx::query_as!(
        OrderModel,
        "select * from orders offset $1 limit $2",
        offset as i64,
        page_size as i64
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    let mut customer_ids = orders
        .iter()
        .map(|order| order.customer_id)
        .collect::<Vec<i32>>();
    customer_ids.dedup();
    tracing::info!("customer_ids: {:?}", customer_ids);

    let customer_ids_joined = customer_ids
        .iter()
        .map(|id| id.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let customers = sqlx::query_as::<_, CustomerModel>(&format!(
        "select * from customers where id in ({customer_ids_joined})"
    ))
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;
    tracing::info!("customers found: {}", customers.len());

    let id_customer = customers
        .iter()
        .map(|customer| (customer.id, customer.clone()))
        .collect::<HashMap<_, _>>();
    tracing::info!("id_customer: {:?}", id_customer);

    let order_dtos = orders
        .iter()
        .map(|order| {
            if let Some(customer) = id_customer.get(&order.customer_id) {
                OrderDto::from(order.clone(), customer.clone())
            } else {
                OrderDto::from_only(order.clone())
            }
        })
        .collect::<Vec<OrderDto>>();

    let count = sqlx::query!("select count(1) from orders")
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?
        .count
        .unwrap_or(0);

    Ok(APIListResponse::new(order_dtos, count as i32))
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
            "select * from order_items where order_id = {} order by id desc offset {} limit {}",
            self.order_id, offset, page_size
        )
    }

    fn to_count_sql(&self) -> String {
        format!(
            "select count(1) from order_items where order_id = {}",
            self.order_id
        )
    }
}

async fn get_order_items(
    State(state): State<Arc<AppState>>,
    Query(param): Query<OrderItemsQuery>,
) -> ERPResult<APIListResponse<OrderItemModel>> {
    if param.order_id == 0 {
        return Err(ERPError::ParamNeeded(param.order_id.to_string()));
    }

    let order_items = sqlx::query_as::<_, OrderItemModel>(&param.to_pagination_sql())
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    let count: (i64,) = sqlx::query_as(&param.to_count_sql())
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(order_items, count.0 as i32))
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateOrderParam {
    id: i32,
    order_no: String,
    customer_id: i32,
    order_date: i32,
    delivery_date: i32,
}

impl UpdateOrderParam {
    pub fn to_sql(&self) -> String {
        format!("update orders set order_no='{}', customer_id={}, order_date={}, delivery_date={} where id={};", self.order_no, self.customer_id, self.order_date, self.delivery_date, self.id)
    }
}

async fn update_order(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateOrderParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let order = sqlx::query_as!(OrderModel, "select * from orders where id = $1", payload.id)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::NotFound(format!("Order#{} {err}", payload.id)))?;

    let _ = sqlx::query(&payload.to_sql())
        .execute(&state.db)
        .await
        .map_err(ERPError::DBError)?;

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
    WithRejection(Json(param), _): WithRejection<Json<UpdateOrderItemParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let _ = sqlx::query_as!(
        OrderItemModel,
        "select * from order_items where id = $1",
        param.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| ERPError::NotFound(format!("OrderItem#{} {err}", param.id)))?;

    sqlx::query(&param.to_sql())
        .execute(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize, Serialize)]
struct ListOrderItemMaterialsParam {
    pub order_id: Option<i32>,
    pub order_item_id: i32,
    pub name: Option<String>,
    pub color: Option<String>,
    pub page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    pub page_size: Option<i32>,
}

impl ListParamToSQLTrait for ListOrderItemMaterialsParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = "select * from order_item_materials".to_string();
        let mut where_clauses = vec![];
        if let Some(order_id) = self.order_id {
            where_clauses.push(format!("order_id={}", order_id));
        }
        where_clauses.push(format!("order_item_id={}", self.order_item_id));
        if let Some(name) = &self.name {
            where_clauses.push(format!("name='{}'", name));
        }
        if let Some(color) = &self.color {
            where_clauses.push(format!("color='{}'", color));
        }
        sql.push_str(" where ");
        sql.push_str(&where_clauses.join(" and "));

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        sql.push_str(&format!(
            " order by id desc limit {page} offset {page_size};"
        ));

        tracing::info!("{sql}");
        sql
    }

    fn to_count_sql(&self) -> String {
        let mut sql = "select count(1) from order_item_materials".to_string();
        let mut where_clauses = vec![];
        if let Some(order_id) = self.order_id {
            where_clauses.push(format!("order_id={}", order_id));
        }
        where_clauses.push(format!("order_item_id={}", self.order_item_id));
        if let Some(name) = &self.name {
            where_clauses.push(format!("name='{}'", name));
        }
        if let Some(color) = &self.color {
            where_clauses.push(format!("color='{}'", color));
        }
        sql.push_str(" where ");
        sql.push_str(&where_clauses.join(" and "));
        sql.push(';');

        tracing::info!("{sql}");
        sql
    }
}

async fn get_order_item_materials(
    State(state): State<Arc<AppState>>,
    Query(param): Query<ListOrderItemMaterialsParam>,
) -> ERPResult<APIListResponse<OrderItemMaterialModel>> {
    let materials = sqlx::query_as::<_, OrderItemMaterialModel>(&param.to_pagination_sql())
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    let total: (i64,) = sqlx::query_as(&param.to_count_sql())
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(materials, total.0 as i32))
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CreateOrderItemMaterialParam {
    pub order_id: i32,
    pub order_item_id: i32,
    pub name: String,
    pub color: String,
    // material_id   integer, -- 材料ID  (暂时先不用)
    pub single: Option<i32>,
    //  integer, -- 单数      ？小数
    pub count: i32,
    //  integer, -- 数量      ？小数
    pub total: Option<i32>,
    //  integer, -- 总数(米)  ? 小数
    pub stock: Option<i32>,
    //  integer, -- 库存 ?
    pub debt: Option<i32>,
    //  integer, -- 欠数
    pub notes: Option<String>, //     text     -- 备注
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct CreateOrderItemMaterialsParam {
    order_item_id: i32,
    materials: Vec<CreateOrderItemMaterialParam>,
}

impl CreateOrderItemMaterialsParam {
    fn to_sql(&self) -> String {
        let values = self
            .materials
            .iter()
            .map(|material| {
                format!(
                    "({}, {}, '{}', '{}', {:?}, {}, {:?}, {:?}, {:?}, '{:?}')",
                    material.order_id,
                    material.order_item_id,
                    material.name,
                    material.color,
                    material.single,
                    material.count,
                    material.total,
                    material.stock,
                    material.debt,
                    material.notes
                )
            })
            .collect::<Vec<String>>()
            .join(",");

        format!("insert into order_item_materials (order_id, order_item_id, name, color, single, count, total, stock, debt, notes) values {};", values)
    }
}

async fn add_order_item_materials(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateOrderItemMaterialsParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    // checking material is empty
    if payload.materials.is_empty() {
        return Err(ERPError::ParamNeeded("materials".to_string()));
    }

    // checking material
    let existings = sqlx::query_as::<_, OrderItemMaterialModel>(&format!(
        "select * from order_item_materials where order_item_id={};",
        payload.order_item_id
    ))
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    // if already have some material, then check against it.
    if !existings.is_empty() {
        let existing_name_color_tuples: Vec<(String, String)> = existings
            .iter()
            .map(|material| (material.name.clone(), material.color.clone()))
            .collect();

        let duplicates = payload
            .materials
            .iter()
            .filter(|&material| {
                existing_name_color_tuples
                    .contains(&(material.name.clone(), material.color.clone()))
            })
            .map(|material| format!("({}-{})", material.name, material.color))
            .collect::<Vec<String>>();

        if !duplicates.is_empty() {
            return Err(ERPError::AlreadyExists(duplicates.join(",")));
        }
    }

    sqlx::query(&payload.to_sql())
        .execute(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct UpdateOrderItemMaterialParam {
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

impl UpdateOrderItemMaterialParam {
    fn to_sql(&self) -> String {
        let mut sql = format!(
            "update order_items set order_id={},sku_id={},count={}",
            self.order_id, self.sku_id, self.count
        );
        if let Some(package_card) = &self.package_card {
            sql.push_str(&format!(",package_card='{}'", package_card));
        }
        if let Some(package_card_des) = &self.package_card_des {
            sql.push_str(&format!(",package_card_des='{}'", package_card_des));
        }
        if let Some(unit) = &self.unit {
            sql.push_str(&format!(",unit='{}'", unit));
        }
        if let Some(unit_price) = &self.unit_price {
            sql.push_str(&format!(",unit_price={}", unit_price));
        }
        if let Some(total_price) = &self.total_price {
            sql.push_str(&format!(",total_price='{}'", total_price));
        }
        if let Some(notes) = &self.notes {
            sql.push_str(&format!(",notes={}", notes));
        }

        sql
    }
}

async fn update_order_item_material(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateOrderItemParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let _ = sqlx::query_as!(
        OrderItemModel,
        "select * from order_items where id=$1",
        payload.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(|err| ERPError::NotFound(format!("OrderItem#{} {err}", payload.id)))?;

    sqlx::query(&payload.to_sql())
        .execute(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIEmptyResponse::new())
}

#[cfg(test)]
mod tests {
    use crate::handler::routes_order::CreateOrderParam;
    use anyhow::Result;
    use tokio;

    #[tokio::test]
    async fn test() -> Result<()> {
        let param = CreateOrderParam {
            customer_id: 12,
            order_no: "order_no".to_string(),
            order_date: 0,
            delivery_date: 0,
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
