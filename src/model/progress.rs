use chrono::NaiveDateTime;

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct ProgressModel {
    pub id: i32, // SERIAL,
    // pub order_id: i32,      // 订单ID
    pub order_item_id: i32, // 订单商品ID
    pub step: i32,          // 当前是第几步
    pub account_id: i32,    // 操作人ID
    pub done: bool,         // 完成
    pub notes: String,      // 备注
    pub dt: NaiveDateTime,  // 操作日期
}
