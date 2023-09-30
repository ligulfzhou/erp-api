use crate::dto::dto_goods::SKUModelDto;
use crate::model::goods::GoodsModel;

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
    pub goods: GoodsModel,
    pub skus: Vec<ReturnOrderItemStat>,
}

#[derive(Serialize)]
pub struct ReturnOrderStatByGoods {
    pub count: i32,
    pub sum: i32,
    pub goods_stat: Vec<ReturnOrderGoodsStat>,
}
