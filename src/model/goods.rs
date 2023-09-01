use crate::{ERPError, ERPResult};
use sqlx::{Pool, Postgres};

#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct GoodsModel {
    pub id: i32,               // SERIAL,
    pub goods_no: String,      // 类目编号
    pub image: String,         // 图片
    pub name: String,          // 名称
    pub plating: String,       // 电镀
    pub notes: Option<String>, // 备注
}

impl GoodsModel {
    pub async fn get_goods_with_goods_no(
        db: &Pool<Postgres>,
        goods_no: &str,
    ) -> ERPResult<Option<GoodsModel>> {
        let goods = sqlx::query_as::<_, GoodsModel>(&format!(
            "select * from goods where goods_no='{}'",
            goods_no
        ))
        .fetch_optional(db)
        .await
        .map_err(ERPError::DBError)?;

        Ok(goods)
    }

    pub async fn insert_goods_to_db(db: &Pool<Postgres>, goods: &GoodsModel) -> ERPResult<i32> {
        let sql = format!(
            r#"insert into goods (goods_no, name, image)
            values ('{}', '{}', '{}')
            returning id"#,
            goods.goods_no, goods.name, goods.image
        );
        let (goods_id,) = sqlx::query_as::<_, (i32,)>(&sql)
            .fetch_one(db)
            .await
            .map_err(ERPError::DBError)?;
        Ok(goods_id)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SKUModel {
    pub id: i32,
    pub goods_id: i32, // 产品ID
    // pub image: Option<String>,   // 商品图片
    // pub plating: Option<String>, // 电镀
    pub sku_no: Option<String>, // sku no (这个只有L1005的有)
    pub color: String,          // 颜色
    pub color2: Option<String>, // 颜色2
    pub notes: Option<String>,  // 备注
}

impl SKUModel {
    pub async fn get_skus_with_goods_id(
        db: &Pool<Postgres>,
        goods_id: i32,
    ) -> ERPResult<Vec<SKUModel>> {
        let sql = format!("select * from skus where goods_id={}", goods_id);
        let skus = sqlx::query_as::<_, SKUModel>(&sql)
            .fetch_all(db)
            .await
            .map_err(ERPError::DBError)?;
        Ok(skus)
    }

    pub async fn get_sku_with_goods_id_and_color(
        db: &Pool<Postgres>,
        goods_id: i32,
        color: &str,
    ) -> ERPResult<Option<SKUModel>> {
        let sql = format!(
            "select * from skus where goods_id={} and color='{}';",
            goods_id, color
        );
        let sku = sqlx::query_as::<_, SKUModel>(&sql)
            .fetch_optional(db)
            .await
            .map_err(ERPError::DBError)?;
        Ok(sku)
    }
}
