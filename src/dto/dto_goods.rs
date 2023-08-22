use crate::model::goods::{GoodsModel, SKUModel};

#[derive(Debug, Deserialize, Serialize)]
pub struct SKUModelDto {
    pub id: i32,
    pub goods_no: String,        // 产品编号 (暂时没有)
    pub image: Option<String>,   // 商品图片
    pub plating: Option<String>, // 电镀
    pub color: Option<String>,   // 颜色
    pub notes: Option<String>,   // 备注
}

impl SKUModelDto {
    pub fn from(sku: SKUModel) -> SKUModelDto {
        Self {
            id: sku.id,
            goods_no: sku.goods_no,
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
    pub goods_no: String, // 类目编号
    pub image: String,    // 图片
    pub name: String,     // 名称
    pub notes: String,    // 备注
    pub skus: Vec<SKUModelDto>,
}

impl GoodsDto {
    pub fn from(goods: GoodsModel, skus: Vec<SKUModel>) -> GoodsDto {
        let skus_dtos = skus
            .iter()
            .map(|sku| SKUModelDto::from(sku.clone()))
            .collect::<Vec<SKUModelDto>>();
        Self {
            id: goods.id,
            goods_no: goods.goods_no,
            image: goods.image,
            name: goods.name,
            notes: goods.notes.unwrap_or("".to_string()),
            skus: skus_dtos,
        }
    }
}