#[derive(Debug, Deserialize, Serialize, Clone, sqlx::FromRow)]
pub struct GoodsModel {
    pub id: i32,               // SERIAL,
    pub goods_no: String,      // 类目编号
    pub image: String,         // 图片
    pub name: String,          // 名称
    pub notes: Option<String>, // 备注
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::FromRow)]
pub struct SKUModel {
    pub id: i32,
    pub goods_no: String,        // 产品编号 (暂时没有)
    pub image: Option<String>,   // 商品图片
    pub plating: Option<String>, // 电镀
    pub color: Option<String>,   // 颜色
    pub notes: Option<String>,   // 备注
}
