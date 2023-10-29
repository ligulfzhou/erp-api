use crate::dto::dto_goods::{GoodsDto, SKUModelDto};

#[derive(Serialize)]
pub struct ReturnOrderStat {
    pub sku: SKUModelDto,
    pub count: i32,
    pub sum: i32,
}

#[derive(Serialize)]
pub struct ReturnOrderItemStat {
    pub sku: SKUModelDto,
    pub count: i32,
    pub sum: i32,
}

#[derive(Serialize)]
pub struct ReturnOrderGoodsStat {
    pub goods: GoodsDto,
    pub skus: Vec<ReturnOrderItemStat>,
    pub count: i32,
    pub sum: i32,
}
