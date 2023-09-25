use crate::model::goods::{GoodsModel, SKUModel};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct SKUModelDto {
    pub id: i32,
    pub sku_no: String,
    pub customer_no: String,
    pub name: String,
    pub goods_no: String,      // 产品编号 (暂时没有)
    pub goods_id: i32,         // 产品ID
    pub image: Option<String>, // 商品图片
    pub plating: String,       // 电镀
    pub color: String,         // 颜色
    pub color2: String,
    pub notes: Option<String>, // 备注
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoodsDto {
    pub id: i32,
    pub goods_no: String,    // 商品编号
    pub image: String,       // 图片
    pub name: String,        // 名称
    pub notes: String,       // 备注
    pub count: i32,          // 多少件商品
    pub skus: Vec<SKUModel>, // 该类目下的所有sku
}

impl GoodsDto {
    pub fn from(goods: GoodsModel, skus: Vec<SKUModel>) -> GoodsDto {
        Self {
            id: goods.id,
            goods_no: goods.goods_no,
            image: goods.image,
            name: goods.name,
            notes: goods.notes,
            count: skus.len() as i32,
            skus,
        }
    }
}
