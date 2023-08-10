#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct OrderModel {
    pub id: i32,
    pub customer_id: i32,
    pub order_no: String,
    pub order_date: i32,
    pub delivery_date: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemModel {
    pub id: i32,
    pub order_id: i32,
    pub sku_id: i32,
    pub package_card: Option<String>,
    pub package_card_des: Option<String>,
    pub count: Option<i32>,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderItemMaterialModel {
    pub id: i32,
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
