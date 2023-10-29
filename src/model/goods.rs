#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct GoodsModel {
    pub id: i32,             // SERIAL,
    pub customer_no: String, // 客户ID
    pub goods_no: String,    // 类目编号
    pub name: String,        // 名称
    pub plating: String,     // 电镀
    pub notes: String,       // 备注
}

// impl GoodsModel {
// pub async fn get_goods_with_goods_no(
//     db: &Pool<Postgres>,
//     goods_no: &str,
// ) -> ERPResult<Option<GoodsModel>> {
//     let goods = sqlx::query_as!(
//         GoodsModel,
//         "select * from goods where goods_no=$1",
//         goods_no
//     )
//     .fetch_optional(db)
//     .await
//     .map_err(ERPError::DBError)?;
//
//     Ok(goods)
// }

// pub async fn insert_goods_to_db(
//     db: &Pool<Postgres>,
//     goods: &GoodsModel,
//     customer_no: &str,
// ) -> ERPResult<i32> {
//     let id = sqlx::query!(
//         r#"
//         insert into goods (goods_no, name, customer_no)
//         values ($1, $2, $3) returning id"#,
//         goods.goods_no,
//         goods.name,
//         customer_no
//     )
//     .fetch_one(db)
//     .await
//     .map_err(ERPError::DBError)?
//     .id;
//
//     Ok(id)
// }
// }

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SKUModel {
    pub id: i32,
    pub goods_id: i32,         // 产品ID
    pub sku_no: String,        // sku no (这个只有L1005的有)
    pub color: String,         // 颜色
    pub color2: String,        // 颜色2
    pub notes: Option<String>, // 备注
}

// impl SKUModel {
//     pub async fn get_skus_with_goods_id(
//         db: &Pool<Postgres>,
//         goods_id: i32,
//     ) -> ERPResult<Vec<SKUModel>> {
//         let skus = sqlx::query_as!(SKUModel, "select * from skus where goods_id=$1", goods_id)
//             .fetch_all(db)
//             .await
//             .map_err(ERPError::DBError)?;
//
//         Ok(skus)
//     }
//
//     pub async fn get_sku_with_goods_id_and_color(
//         db: &Pool<Postgres>,
//         goods_id: i32,
//         color: &str,
//     ) -> ERPResult<Option<SKUModel>> {
//         let sku = sqlx::query_as!(
//             SKUModel,
//             "select * from skus where goods_id=$1 and color=$2;",
//             goods_id,
//             color
//         )
//         .fetch_optional(db)
//         .await
//         .map_err(ERPError::DBError)?;
//         Ok(sku)
//     }
// }
