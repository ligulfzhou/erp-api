#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct GoodsModel {
    // SERIAL,
    pub id: i32,
    // 类目编号
    pub goods_no: Option<String>,
    // 图片
    pub image: Option<String>,
    // 名称
    pub name: Option<String>,
    // 电镀
    pub plating: Option<String>,
    // 备注
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct SKUModel {
    pub id: i32,

    pub sku_no: String,
    // 类目ID
    pub goods_id: Option<i32>,
    //商品图片
    pub image: Option<String>,
    // 产品编号 (暂时没有)
    pub goods_no: String,
    // 颜色
    pub color: Option<String>,
    // 备注
    pub notes: Option<String>,
}
