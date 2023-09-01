use crate::constants::DEFAULT_PAGE_SIZE;
use crate::dto::dto_customer::CustomerDto;
use crate::handler::ListParamToSQLTrait;
use crate::model::customer::CustomerModel;
use crate::response::api_response::{APIDataResponse, APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/customers", get(get_customers).post(create_customer))
        .route("/api/customer/detail", get(detail_customer))
        .route("/api/customer/update", post(update_customer))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct ListCustomerParam {
    name: Option<String>,
    customer_no: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl ListParamToSQLTrait for ListCustomerParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = "select * from customers ".to_string();

        let mut where_clauses = vec![];
        if self.name.is_some() && !self.name.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("name like '%{}%'", self.name.as_ref().unwrap()));
        }
        if self.customer_no.is_some() && !self.customer_no.as_ref().unwrap().is_empty() {
            where_clauses.push(format!(
                "customer_no='{}'",
                self.customer_no.as_ref().unwrap()
            ));
        }
        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(
            " order by id desc offset {} limit {};",
            offset, page_size
        ));

        tracing::info!("get_customer_list sql: {sql}");

        sql
    }

    fn to_count_sql(&self) -> String {
        let mut sql = "select count(1) from customers ".to_string();

        let mut where_clauses = vec![];
        if self.name.is_some() && !self.name.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("name like '%{}%'", self.name.as_ref().unwrap()));
        }
        if self.customer_no.is_some() && !self.customer_no.as_ref().unwrap().is_empty() {
            where_clauses.push(format!(
                "customer_no='{}'",
                self.customer_no.as_ref().unwrap()
            ));
        };

        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }
        sql.push(';');

        tracing::info!("get_customer_count sql: {sql}");
        sql
    }
}

async fn get_customers(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<ListCustomerParam>, ERPError>,
) -> ERPResult<APIListResponse<CustomerDto>> {
    let pagination_sql = param.to_pagination_sql();
    let customers = sqlx::query_as::<_, CustomerModel>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;
    let customer_dtos = customers
        .iter()
        .map(|customer| CustomerDto::from(customer.clone()))
        .collect();

    let count_sql = param.to_count_sql();
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(customer_dtos, total.0 as i32))
}

#[derive(Debug, Deserialize)]
struct CreateCustomerParam {
    pub customer_no: String,
    pub name: Option<String>,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

impl CreateCustomerParam {
    fn to_sql(&self) -> String {
        format!(
            "insert into customers (customer_no, name, address, phone, notes) values ('{}', '{}', '{}', '{}', '{}')",
            self.customer_no, self.name.as_ref().unwrap_or(&"".to_string()), self.address.as_ref().unwrap_or(&"".to_string()), self.phone.as_ref().unwrap_or(&"".to_string()), self.notes.as_ref().unwrap_or(&"".to_string())
        )
    }
}

async fn create_customer(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<CreateCustomerParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let customer = sqlx::query_as::<_, CustomerModel>(&format!(
        "select * from customers where customer_no = '{}'",
        payload.customer_no
    ))
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if customer.is_some() {
        return Err(ERPError::AlreadyExists(format!(
            "客户ID#{}已存在",
            payload.customer_no
        )));
    }

    let sql = payload.to_sql();
    sqlx::query(&sql)
        .execute(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct DetailParam {
    id: i32,
}

async fn detail_customer(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<DetailParam>, ERPError>,
) -> ERPResult<APIDataResponse<CustomerModel>> {
    let customer = sqlx::query_as::<_, CustomerModel>(&format!(
        "select * from customers where id = {};",
        param.id
    ))
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if customer.is_none() {
        return Err(ERPError::NotFound(format!("Customer#{}", param.id)));
    }

    Ok(APIDataResponse::new(customer.unwrap()))
}

#[derive(Debug, Deserialize)]
struct UpdateCustomerParam {
    pub id: i32,
    pub customer_no: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

impl UpdateCustomerParam {
    fn to_sql(&self) -> String {
        let mut set_clauses = vec![];
        set_clauses.push(format!(
            "customer_no='{}',name='{}'",
            self.customer_no, self.name
        ));
        if let Some(address) = &self.address {
            set_clauses.push(format!("address='{}'", address))
        }
        if let Some(phone) = &self.phone {
            set_clauses.push(format!("phone='{}'", phone))
        }
        if let Some(notes) = &self.notes {
            set_clauses.push(format!("notes='{}'", notes))
        }

        format!(
            "update customers set {} where id = {};",
            set_clauses.join(","),
            self.id
        )
    }
}

async fn update_customer(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateCustomerParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let customer = sqlx::query_as::<_, CustomerModel>(&format!(
        "select * from customers where id = {}",
        payload.id
    ))
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if customer.customer_no != payload.customer_no
        && sqlx::query_as::<_, CustomerModel>(&format!(
            "select * from customers where customer_no='{}'",
            payload.customer_no
        ))
        .fetch_optional(&state.db)
        .await
        .map_err(ERPError::DBError)
        .is_ok()
    {
        return Err(ERPError::Collision(format!(
            "{} 已存在",
            payload.customer_no
        )));
    }

    sqlx::query(&payload.to_sql()).execute(&state.db).await?;

    Ok(APIEmptyResponse::new())
}

#[cfg(test)]
mod tests {

    #[test]
    fn test() {}
}
