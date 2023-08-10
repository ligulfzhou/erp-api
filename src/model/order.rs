#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OrderModel {
    pub id: i32,
    pub customer_id: Option<i32>,
    pub order_no: Option<String>,
    pub order_date: Option<i32>,
    pub delivery_date: Option<i32>,
    // pub delivery_date_str: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemModel {
    pub id: i32,
    pub order_id: Option<i32>,
    pub sku_id: Option<i32>,
    pub package_card: Option<String>,
    pub package_card_des: Option<String>,
    pub count: Option<i32>,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes: Option<String>,
}
