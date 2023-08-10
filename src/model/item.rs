use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemModel {
    pub id: i32,
    pub num: Option<i32>,
    pub image: Option<String>,
    pub size: Option<String>,
    pub buy_price: Option<i32>,
    pub sell_price: Option<i32>,
    pub memo: Option<String>,
}
