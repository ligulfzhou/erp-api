use crate::dto::dto_goods::SKUModelDto;

#[derive(Serialize)]
pub struct ReturnOrderStat {
    pub sku: SKUModelDto,
    pub count: i32,
    pub sum: i32,
}
