#[derive(Debug, Deserialize, Serialize)]
pub struct GoodsModel {
    // SERIAL,
    id: i32,
    // 类目编号
    goods_no: Option<String>,
    // 图片
    image: Option<String>,
    // 名称
    name: Option<String>,
    // 电镀
    plating: Option<String>,
    // 备注
    notes: Option<String>,
}
