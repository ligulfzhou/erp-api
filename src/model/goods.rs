#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct GoodsModel {
    pub id: i32,                  // SERIAL,
    pub goods_no: Option<String>, // 类目编号
    pub image: Option<String>,    // 图片
    pub name: Option<String>,     // 名称
    pub plating: Option<String>,  // 电镀
    pub notes: Option<String>,    // 备注
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SKUModel {
    pub id: i32,
    pub sku_no: String,          // sku编号
    pub goods_id: Option<i32>,   // 类目ID
    pub goods_no: String,        // 产品编号 (暂时没有)
    pub name: Option<String>,    // 商品名
    pub image: Option<String>,   // 商品图片
    pub plating: Option<String>, // 电镀
    pub color: Option<String>,   // 颜色
    pub notes: Option<String>,   // 备注
}
