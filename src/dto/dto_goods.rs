use crate::model::goods::{GoodsModel, SKUModel};
use crate::ERPResult;
use sqlx::{FromRow, Pool, Postgres};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone)]
pub struct GoodsImagesAndPackage {
    pub images: Vec<String>,
    pub image_des: String,
    pub package_card: String,
    pub package_card_des: String,
    pub goods_id: i32,
}

#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct SKUModelWithoutImageAndPackageDto {
    pub id: i32,
    pub sku_no: String,
    pub customer_no: String,
    pub name: String,
    pub goods_no: String, // 产品编号 (暂时没有)
    pub goods_id: i32,    // 产品ID
    // pub images: Vec<String>,  // 商品图片
    // pub image_des: String,
    // pub package_card: String, // 标签图片
    // pub package_card_des: String, // 标签说明
    pub plating: String, // 电镀
    pub color: String,   // 颜色
    pub color2: String,
    pub notes: Option<String>, // 备注
}

#[derive(Debug, Deserialize, Serialize, FromRow, Clone)]
pub struct SKUModelDto {
    pub id: i32,
    pub sku_no: String,
    pub customer_no: String,
    pub name: String,
    pub goods_no: String,         // 产品编号 (暂时没有)
    pub goods_id: i32,            // 产品ID
    pub images: Vec<String>,      // 商品图片
    pub image_des: String,        // 商品图片描述
    pub package_card: String,     // 标签图片
    pub package_card_des: String, // 标签说明
    pub plating: String,          // 电镀
    pub color: String,            // 颜色
    pub color2: String,
    pub notes: Option<String>, // 备注
}

impl SKUModelDto {
    pub fn from_sku_and_images_package(
        sku: SKUModelWithoutImageAndPackageDto,
        images_package: GoodsImagesAndPackage,
    ) -> SKUModelDto {
        Self {
            id: sku.id,
            sku_no: sku.sku_no,
            customer_no: sku.customer_no,
            name: sku.name,
            goods_no: sku.goods_no,
            goods_id: sku.goods_id,
            images: images_package.images,
            image_des: images_package.image_des,
            package_card: images_package.package_card,
            package_card_des: images_package.package_card_des,
            plating: sku.plating,
            color: sku.color,
            color2: sku.color2,
            notes: sku.notes,
        }
    }

    pub fn get_skus_with_goods_ids(
        db: &Pool<Postgres>,
        goods_ids: &[i32],
    ) -> ERPResult<Vec<SKUModelDto>> {
        todo!()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GoodsDto {
    pub id: i32,
    pub goods_no: String,         // 商品编号
    pub customer_no: String,      // 客户ID
    pub images: Vec<String>,      // 图片
    pub image_des: String,        // 图片描述
    pub name: String,             // 名称
    pub plating: String,          // 电镀
    pub package_card: String,     // 标签图片
    pub package_card_des: String, // 标签说明
    pub notes: String,            // 备注
    pub count: i32,               // 多少件商品
    pub skus: Vec<SKUModel>,      // 该类目下的所有sku
}

impl GoodsDto {
    pub fn from_goods_and_images_package(
        goods: GoodsModel,
        images_package: GoodsImagesAndPackage,
    ) -> GoodsDto {
        Self {
            id: goods.id,
            goods_no: goods.goods_no,
            customer_no: goods.customer_no,
            images: images_package.images,
            image_des: images_package.image_des,
            name: goods.name,
            plating: goods.plating,
            package_card: images_package.package_card,
            package_card_des: images_package.package_card_des,
            notes: goods.notes,
            count: 0,
            skus: vec![],
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GoodsImageAndPackage {
    pub goods_no: String,
    pub images: Vec<String>,
    pub image_des: String,
    pub package_card: String,
    pub package_card_des: String,
}

impl GoodsDto {
    pub fn from(goods: GoodsModel, skus: Vec<SKUModel>) -> GoodsDto {
        Self {
            id: goods.id,
            goods_no: goods.goods_no,
            customer_no: goods.customer_no,
            // images: goods.images,
            // image_des: goods.image_des,
            images: vec![],
            image_des: "".to_string(),
            name: goods.name,
            plating: goods.plating,
            // package_card: goods.package_card,
            // package_card_des: goods.package_card_des,
            package_card: "".to_string(),
            package_card_des: "".to_string(),
            notes: goods.notes,
            count: skus.len() as i32,
            skus,
        }
    }
}
