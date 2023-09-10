use crate::constants::DEFAULT_PAGE_SIZE;
use crate::dto::dto_account::AccountDto;
use crate::dto::dto_orders::{
    OrderDto, OrderGoodsDto, OrderGoodsItemDto, OrderGoodsItemWithStepsDto,
    OrderGoodsWithStepsWithItemStepDto, OrderWithStepsDto,
};
use crate::dto::dto_progress::OneProgress;
use crate::handler::ListParamToSQLTrait;
use crate::middleware::auth::auth;
use crate::model::order::OrderModel;
use crate::model::progress::ProgressModel;
use crate::response::api_response::{APIDataResponse, APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{middleware, Extension, Json, Router};
use axum_extra::extract::WithRejection;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/orders", get(get_orders).post(create_order))
        .route("/api/orders/dates", get(get_orders_dates))
        .route("/api/order/detail", get(order_detail))
        .route("/api/order/update", post(update_order))
        .route("/api/order/items", get(get_order_items))
        .route("/api/order/goods/update", post(update_order_goods))
        .route("/api/order/item/update", post(update_order_item))
        .route("/api/order/goods/delete", post(delete_order_goods))
        .route("/api/order/item/delete", post(delete_order_item))
        .route_layer(middleware::from_fn_with_state(state.clone(), auth))
        .with_state(state)
}

#[derive(Deserialize)]
struct OrderDatesParam {
    customer_no: String,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

#[derive(Debug, FromRow, Serialize)]
struct OrderDates {
    order_no: String,
    order_date: NaiveDate,
}

async fn get_orders_dates(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<OrderDatesParam>, ERPError>,
) -> ERPResult<APIListResponse<OrderDates>> {
    let page = param.page.unwrap_or(1);
    let page_size = param.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = (page - 1) * page_size;
    let customer_no = param.customer_no;

    let dates = sqlx::query_as!(
        OrderDates,
        r#"
        select order_no, order_date
        from orders
        where customer_no = $1
            order by order_date desc
        offset $2 limit $3;
        "#,
        customer_no,
        offset as i64,
        page_size as i64
    )
    .fetch_all(&state.db)
    .await
    .map_err(|_| ERPError::Failed("数据库错误，获取订单日期列表失败".to_string()))?;

    let count = sqlx::query!(
        r#"
        select count(1)
        from orders
        where customer_no = $1;
        "#,
        customer_no
    )
    .fetch_one(&state.db)
    .await
    .map_err(|_| ERPError::Failed("数据库错误，获取订单日期数量失败".to_string()))?
    .count
    .unwrap_or(0) as i32;

    Ok(APIListResponse::new(dates, count))
}

#[derive(Debug, Deserialize)]
struct DeleteOrderItemParam {
    id: i32,
}

async fn delete_order_item(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<DeleteOrderItemParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    sqlx::query!("delete from order_items where id = $1", payload.id)
        .execute(&state.db)
        .await
        .map_err(|_| ERPError::Failed("删除数据失败".to_string()))?;
    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct DeleteOrderGoods {
    id: i32,
}

async fn delete_order_goods(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<DeleteOrderGoods>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    // 先删 order_goods
    sqlx::query!("delete from order_goods where id = $1", payload.id)
        .execute(&state.db)
        .await
        .map_err(|_| ERPError::Failed("删除数据失败".to_string()))?;

    // 再删 order_items
    sqlx::query!(
        "delete from order_items where order_goods_id=$1",
        payload.id
    )
    .execute(&state.db)
    .await
    .map_err(|_| ERPError::Failed("删除数据失败".to_string()))?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateOrderParam {
    customer_no: String,
    order_no: String,
    order_date: NaiveDate,
    delivery_date: Option<NaiveDate>,
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

    sqlx::query!(
        r#"
        insert into orders (customer_no, order_no, order_date, delivery_date, is_urgent, is_return_order)
        values ($1, $2, $3, $4, $5, $6)
        "#, payload.customer_no, payload.order_no, payload.order_date, payload.delivery_date, payload.is_urgent, payload.is_return_order
    ).execute(&state.db).await?;

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
    let id = param.id.unwrap_or(0);
    let order_no = param.order_no.as_deref().unwrap_or("");

    let order_dto = match id {
        0 => sqlx::query_as!(OrderDto, "select * from orders where order_no=$1", order_no)
            .fetch_one(&state.db)
            .await
            .map_err(ERPError::DBError)?,
        _ => sqlx::query_as!(OrderDto, "select * from orders where id=$1", id)
            .fetch_one(&state.db)
            .await
            .map_err(ERPError::DBError)?,
    };

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

        let mut sql = "select * from orders".to_string();
        let mut where_clauses = vec![];

        let customer_no = self.customer_no.as_deref().unwrap_or("");
        if !customer_no.is_empty() {
            where_clauses.push(format!("customer_no='{}'", customer_no));
        }
        let order_no = self.order_no.as_deref().unwrap_or("");
        if !order_no.is_empty() {
            where_clauses.push(format!("order_no='{}'", customer_no));
        }

        if !where_clauses.is_empty() {
            sql.push_str(&format!(" where {}", where_clauses.join(" and ")))
        }

        sql.push_str(&format!(
            " order by id desc offset {} limit {};",
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
                "select count(1) from orders where order_no = '{}';",
                order_no
            )
        } else if !customer_no.is_empty() && order_no.is_empty() {
            format!(
                r#"
                select count(1)
                from orders
                where customer_no = '{}'
                "#,
                customer_no
            )
        } else {
            format!(
                r#"
                select count(1)
                from orders
                where customer_no = '{}' and order_no = '%{}%'
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
    tracing::info!("{:#?}", order_items_steps);

    let empty_order_item_step = HashMap::new();
    let order_with_step_dtos = order_dtos
        .into_iter()
        .map(|order_dto| {
            let steps = order_items_steps
                .get(&order_dto.id)
                .unwrap_or(&empty_order_item_step);

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

async fn get_order_items(
    Extension(account): Extension<AccountDto>,
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<OrderItemsQuery>, ERPError>,
) -> ERPResult<APIListResponse<OrderGoodsWithStepsWithItemStepDto>> {
    if param.order_id == 0 {
        return Err(ERPError::ParamNeeded(param.order_id.to_string()));
    }

    let page = param.page.unwrap_or(1);
    let page_size = param.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
    let offset = (page - 1) * page_size;

    // 获取order_good
    let order_goods = sqlx::query_as!(
        OrderGoodsDto,
        r#"
        select
            og.id as id, og.order_id as order_id, og.goods_id as goods_id, g.goods_no as goods_no,
            g.name as name, g.image as image, g.plating as plating, g.package_card as package_card,
            g.package_card_des as package_card_des
        from order_goods og, goods g
        where og.goods_id = g.id and og.order_id = $1
        order by og.id offset $2 limit $3
        "#,
        param.order_id,
        offset as i64,
        page_size as i64
    )
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if order_goods.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }
    tracing::info!("order_goods: {:?}, len: {}", order_goods, order_goods.len());

    let order_goods_ids = order_goods.iter().map(|item| item.id).collect::<Vec<i32>>();

    tracing::info!("order_goods_ids: {:?}", order_goods_ids);
    // 用order_goods_ids去获取order_items
    let order_items_dto = sqlx::query_as!(
        OrderGoodsItemDto,
        r#"
        select
            oi.id, oi.order_id, oi.sku_id, s.color, s.sku_no, oi.count, oi.unit,
            oi.unit_price, oi.total_price, oi.notes, og.goods_id
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

    tracing::info!(
        "order_items: {:?}, len: {}",
        order_items_dto,
        order_items_dto.len()
    );

    if order_items_dto.len() <= 0 {
        return Ok(APIListResponse::new(
            order_goods
                .iter()
                .map(|item| {
                    OrderGoodsWithStepsWithItemStepDto::from_order_with_goods_and_steps_and_items(
                        item.clone(),
                        HashMap::new(),
                        vec![],
                        false,
                    )
                })
                .collect::<Vec<OrderGoodsWithStepsWithItemStepDto>>(),
            order_goods.len() as i32,
        ));
    }
    // todo: order_items表应该加一个 order_goods_id 字段
    // let mut order_item_id_to_order_goods_id: HashMap<i32, i32> = HashMap::new();
    // let order_item_id_to_goods_id = order_items_dto
    //     .iter()
    //     .map(|item| (item.id, item.goods_id))
    //     .collect::<HashMap<i32, i32>>();
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
    for one_progress in progresses.into_iter() {
        let progress_vec = order_item_id_to_progress_vec
            .entry(one_progress.order_item_id)
            .or_insert(vec![]);
        progress_vec.push(one_progress);
    }
    tracing::info!(
        "order_item_id_to_progress_vec: {:?}",
        order_item_id_to_progress_vec
    );

    let order_item_id_to_sorted_progress_vec = order_item_id_to_progress_vec
        .iter()
        .map(|oid_progress_vec| (oid_progress_vec.0.clone(), oid_progress_vec.1.clone()))
        .collect::<HashMap<i32, Vec<OneProgress>>>();
    tracing::info!(
        "order_item_id_to_progress_vec after ordering: {:?}",
        order_item_id_to_sorted_progress_vec
    );

    let empty: Vec<OneProgress> = vec![];
    let order_items_with_steps_dtos = order_items_dto
        .into_iter()
        .map(|item| {
            let steps = order_item_id_to_progress_vec
                .get(&item.id)
                .unwrap_or(&empty);

            let step = {
                match steps.len() {
                    0 => 1,
                    _ => match steps[steps.len() - 1].done {
                        true => steps[steps.len() - 1].step + 1,
                        false => steps[steps.len() - 1].step + 0,
                    },
                }
            };
            let is_next_action = account.steps.contains(&step);

            OrderGoodsItemWithStepsDto::from(item, steps.clone(), is_next_action)
        })
        .collect::<Vec<OrderGoodsItemWithStepsDto>>();

    let mut gid_order_item_dtos = HashMap::new();
    for item in order_items_with_steps_dtos.into_iter() {
        let dtos = gid_order_item_dtos.entry(item.goods_id).or_insert(vec![]);
        dtos.push(item);
    }

    let empty_array: Vec<OrderGoodsItemWithStepsDto> = vec![];
    let order_goods_dtos = order_goods
        .into_iter()
        .map(|order_good| {
            let items = gid_order_item_dtos
                .get(&order_good.goods_id)
                .unwrap_or(&empty_array);

            let order_item_step = items
                .iter()
                .map(|item| {
                    let step = {
                        match &item.steps.len() {
                            0 => 1,
                            _ => match &item.steps[item.steps.len() - 1].done {
                                true => &item.steps[item.steps.len() - 1].step + 1,
                                false => &item.steps[item.steps.len() - 1].step + 0,
                            },
                        }
                    };
                    (item.id, step)
                })
                .collect::<HashMap<i32, i32>>();

            let mut order_item_steps_count: HashMap<i32, i32> = HashMap::new();
            for (_, step) in order_item_step.iter() {
                let count = order_item_steps_count.entry(step.to_owned()).or_insert(0);
                *count += 1;
            }

            let mut is_next_action = false;
            let steps = order_item_steps_count
                .iter()
                .map(|sc| sc.0)
                .collect::<Vec<&i32>>();
            if steps.len() == 1 && account.steps.contains(steps[0]) {
                is_next_action = true;
            }
            println!("steps: {:?}, {}", steps, is_next_action);
            OrderGoodsWithStepsWithItemStepDto::from_order_with_goods_and_steps_and_items(
                order_good,
                order_item_steps_count,
                items.clone(),
                is_next_action,
            )
        })
        .collect::<Vec<OrderGoodsWithStepsWithItemStepDto>>();

    let count = sqlx::query!(
        "select count(1) from order_goods where order_id = $1",
        param.order_id
    )
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?
    .count
    .unwrap_or(0) as i32;

    Ok(APIListResponse::new(order_goods_dtos, count))
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateOrderParam {
    id: i32,
    order_no: String,
    customer_no: String,
    order_date: NaiveDate,
    delivery_date: Option<NaiveDate>,
    is_return_order: bool,
    is_urgent: bool,
}

async fn update_order(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateOrderParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let _order = sqlx::query_as!(OrderModel, "select * from orders where id = $1", payload.id)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::NotFound(format!("Order#{} {err}", payload.id)))?;

    sqlx::query!(
        r#"
        update orders set order_no=$1, customer_no=$2, order_date=$3, delivery_date=$4, is_return_order=$5, is_urgent=$6
        where id=$7
        "#, payload.order_no,payload.customer_no,payload.order_date,payload.delivery_date,payload.is_return_order,payload.is_urgent,payload.id
    ).execute(&state.db).await?;

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
    if let Some(_id) = payload.id {
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

        let order_item_id = sqlx::query!(
            "select id from order_items where order_id=$1 and sku_id=$2",
            order_id,
            sku_id
        )
        .fetch_optional(&state.db)
        .await
        .map_err(ERPError::DBError)?;

        if order_item_id.is_some() {
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
    package_card: Option<String>,
    package_card_des: Option<String>,
}

impl UpdateOrderGoodsParam {
    fn to_update_sql(&self) -> String {
        let mut where_clauses = vec![];
        where_clauses.push(format!(
            "package_card='{}'",
            self.package_card.as_deref().unwrap_or("")
        ));
        where_clauses.push(format!(
            "package_card_des='{}'",
            self.package_card_des.as_deref().unwrap_or("")
        ));

        let sql = format!(
            "update order_goods set {} where id={}",
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
    tracing::info!(
        "=> handler update_order_goods: update sql: {:?}",
        payload.to_update_sql()
    );
    state.execute_sql(&payload.to_update_sql()).await?;

    Ok(APIEmptyResponse::new())
}

#[cfg(test)]
mod tests {
    use crate::handler::routes_login::LoginPayload;
    use crate::handler::routes_order::CreateOrderParam;
    use anyhow::Result;
    use chrono::NaiveDate;
    use tokio;

    #[tokio::test]
    async fn test_create_order() -> Result<()> {
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

        let param = CreateOrderParam {
            customer_no: "L1007".to_string(),
            order_no: "order_no_101".to_string(),
            order_date: "2022-03-09".to_string().parse::<NaiveDate>()?,
            delivery_date: None,
            is_urgent: false,
            is_return_order: false,
        };

        client.do_get("/api/orders/dates").await?.print().await?;

        // client
        //     .do_post("/api/orders", serde_json::json!(param))
        //     .await?
        //     .print()
        //     .await?;

        Ok(())
    }
}
