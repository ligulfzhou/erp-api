use sqlx::{Pool, Postgres};
use crate::{ERPError, ERPResult};

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct CustomerModel {
    pub id: i32,
    pub customer_no: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}

impl CustomerModel {
    pub async fn get_customer_with_customer_no(db: &Pool<Postgres>, customer_no: &str)-> ERPResult<CustomerModel> {
        let customer = sqlx::query_as::<_, CustomerModel>(&format!(
            "select id from customers where customer_no='{}'",
            customer_no
        ))
        .fetch_one(db)
        .await
        .map_err(ERPError::DBError)?;

        Ok(customer)
    }
}
