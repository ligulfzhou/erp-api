use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SKU {
    id: i32,
    // 类目ID
    goods_id: i32,
    //商品图片
    image: Option<String>,
    // 产品编号 (暂时没有)
    goods_no: Option<String>,
    // 颜色
    color: Option<String>,
    // 备注
    notes: Option<String>,
}
