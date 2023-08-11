#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct CustomerModel {
    pub id: i32,
    pub customer_no: String,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub notes: Option<String>,
}
