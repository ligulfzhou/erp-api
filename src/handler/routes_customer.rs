use crate::handler::ListParamToSQLTrait;
use crate::model::customer::CustomerModel;
use crate::response::api_response::{APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/customers", get(get_customers).post(create_customer))
        .route("/api/customer/update", post(update_customer))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct ListCustomerParam {
    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl ListParamToSQLTrait for ListCustomerParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = "select * from customers ".to_string();
        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(50);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(" offset {} limit {};", offset, page_size));

        sql
    }

    fn to_count_sql(&self) -> String {
        "select count(1) from customers;".to_string()
    }
}

async fn get_customers(
    State(state): State<Arc<AppState>>,
    Query(list_param): Query<ListCustomerParam>,
) -> ERPResult<APIListResponse<CustomerModel>> {
    let pagination_sql = list_param.to_pagination_sql();
    let customers = sqlx::query_as::<_, CustomerModel>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    let count_sql = list_param.to_count_sql();
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIListResponse::new(customers, total.0 as i32))
}

#[derive(Debug, Deserialize)]
struct CreateCustomerParam {
    pub customer_no: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

impl CreateCustomerParam {
    fn to_sql(&self) -> String {
        format!(
            "insert into customers (customer_no, name, address, phone, notes) values ('{}', '{}', '{}', '{}', '{}')",
            self.customer_no, self.name, self.address.as_ref().unwrap_or(&"".to_string()), self.phone.as_ref().unwrap_or(&"".to_string()), self.notes.as_ref().unwrap_or(&"".to_string())
        )
    }
}

async fn create_customer(
    State(state): State<Arc<AppState>>,
    Json(create_param): Json<CreateCustomerParam>,
) -> ERPResult<APIEmptyResponse> {
    let customer = sqlx::query_as::<_, CustomerModel>(&format!(
        "select * from customers where customer_no = '{}'",
        create_param.customer_no
    ))
    .fetch_optional(&state.db)
    .await
    .map_err(|err| ERPError::DBError(err))?;

    if customer.is_some() {
        return Err(ERPError::AlreadyExists(format!(
            "customer#{}",
            create_param.customer_no
        )));
    }

    let sql = create_param.to_sql();
    sqlx::query(&sql)
        .execute(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct UpdateCustomerParams {
    pub id: i32,
    pub customer_no: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

impl UpdateCustomerParams {
    fn to_sql(&self) -> String {
        let mut set_clauses = vec![];

        set_clauses.push(format!(
            " customer_no='{}', name='{}' ",
            self.customer_no, self.name
        ));
        if let Some(address) = &self.address {
            set_clauses.push(format!(" address = '{}' ", address))
        }
        if let Some(phone) = &self.phone {
            set_clauses.push(format!(" phone = '{}' ", phone))
        }
        if let Some(notes) = &self.notes {
            set_clauses.push(format!(" notes = '{}' ", notes))
        }

        format!(
            "update skus set {} where id = {}",
            set_clauses.join(","),
            self.id
        )
    }
}

async fn update_customer(
    State(state): State<Arc<AppState>>,
    Json(update_param): Json<UpdateCustomerParams>,
) -> ERPResult<APIEmptyResponse> {
    // todo: check customer_no collision
    sqlx::query(&update_param.to_sql())
        .execute(&state.db)
        .await?;

    Ok(APIEmptyResponse::new())
}
