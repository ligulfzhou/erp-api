use crate::model::goods::{GoodsModel, SKUModel};

#[derive(Debug, Deserialize, Serialize)]
pub struct SKUModelDto {
    pub id: i32,
    pub goods_no: String,        // 产品编号 (暂时没有)
    pub goods_id: i32,           // 产品ID
    pub image: Option<String>,   // 商品图片
    pub plating: Option<String>, // 电镀
    pub color: Option<String>,   // 颜色
    pub notes: Option<String>,   // 备注
}

impl SKUModelDto {
    pub fn from(sku: SKUModel) -> SKUModelDto {
        Self {
            id: sku.id,
            goods_id: sku.goods_id,
            goods_no: "todo".to_owned(),
            image: sku.image,
            plating: sku.plating,
            color: sku.color,
            notes: sku.notes,
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
