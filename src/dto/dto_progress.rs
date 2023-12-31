use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Serialize, FromRow, Clone, PartialOrd, PartialEq)]
pub struct OneProgress {
    pub id: i32,
    pub order_item_id: i32,
    pub step: i32,
    pub index: i32,
    pub account_id: i32,
    pub account_name: String,
    pub department: String,
    pub done: bool,
    pub notes: String,
    pub dt: DateTime<Utc>,
}
