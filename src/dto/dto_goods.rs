use crate::model::goods::{GoodsModel, SKUModel};
use sqlx::FromRow;

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct SKUModelDto {
    pub id: i32,
    pub sku_no: String,
    pub name: String,
    pub goods_no: String,      // 产品编号 (暂时没有)
    pub goods_id: i32,         // 产品ID
    pub image: Option<String>, // 商品图片
    pub plating: String,       // 电镀
    pub color: String,         // 颜色
    pub notes: Option<String>, // 备注
}

impl SKUModelDto {
    pub fn from(sku: &SKUModel, goods: &GoodsModel) -> SKUModelDto {
        Self {
            id: sku.id,
            sku_no: sku.sku_no.as_deref().unwrap_or("").to_string(),
            goods_id: sku.goods_id,
            goods_no: goods.goods_no.to_string(),
            image: Some(goods.image.to_owned()),
            plating: goods.plating.to_owned(),
            color: sku.color.to_string(),
            notes: sku.notes.to_owned(),
            name: goods.name.to_string(),
        }
    }
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
            notes: goods.notes.unwrap_or("".to_string()),
            count: skus.len() as i32,
            skus,
        }
    }
}
