use crate::common::hashmap::key_of_max_value;
use crate::common::string::common_prefix;
use crate::model::goods::GoodsModel;
use chrono::NaiveDate;
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone, sqlx::FromRow)]
pub struct OrderModel {
    pub id: i32,
    pub customer_id: i32,
    pub order_no: String,
    pub order_date: NaiveDate,
    pub delivery_date: Option<NaiveDate>,
    // todo: 添加一个“返单，加急配送的”状态字段
    pub is_urgent: bool,       //紧急 ‼️
    pub is_return_order: bool, // 返单
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OrderItemNoIdModel {
    pub order_id: i32,
    pub sku_id: i32,
    // pub goods_id: i32,
    pub package_card: Option<String>,
    pub package_card_des: Option<String>,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes: Option<String>,
}

pub fn multi_order_items_no_id_models_to_sql(models: Vec<OrderItemNoIdModel>) -> String {
    let mut values = vec![];
    for model in models {
        // let unit_price: Option<i32> = None;
        // let total_price: Option<i32> = None;
        values.push(format!(
            "({}, {}, '{}', '{}', {}, '{}', {}, {}, '{}')",
            model.order_id,
            model.sku_id,
            model.package_card.as_ref().unwrap_or(&"".to_string()),
            model.package_card_des.as_ref().unwrap_or(&"".to_string()),
            model.count,
            model.unit.as_ref().unwrap_or(&"".to_string()),
            model.unit_price.as_ref().unwrap_or(&0),
            model.total_price.as_ref().unwrap_or(&0),
            model.notes.as_ref().unwrap_or(&"".to_string())
        ));
    }

    format!("insert into order_items (order_id, sku_id, package_card, package_card_des, count, unit, unit_price, total_price, notes) values {}", values.join(","))
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderGoodsModel {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    // pub order_no: String,
    // pub goods_no: String,
    pub package_card: Option<String>,
    pub package_card_des: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItemModel {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    // pub order_no: String,
    pub sku_id: i32,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExcelOrder {
    pub info: OrderInfo,
    pub items: Vec<OrderItemExcel>,
}

#[derive(Default, Debug, Clone)]
pub struct OrderInfo {
    pub customer_no: String,
    pub order_no: String,
    pub order_date: NaiveDate,
    pub delivery_date: Option<NaiveDate>,
    pub is_return_order: bool,
    pub is_urgent: bool,
}

#[derive(Debug, Default, Clone)]
pub struct OrderItemExcel {
    pub index: i32,
    pub package_card: Option<String>,
    pub package_card_des: Option<String>,
    /// 商品唯一编号
    pub goods_no: String,
    /// 商品编号
    pub goods_no_2: Option<String>, // 反正用处并不大
    /// sku编号 //只有L1005有这个字段
    pub sku_no: Option<String>,
    /// 商品图片
    pub image: Option<String>,
    /// 商品的图片描述
    pub image_des: Option<String>,
    /// 商品名
    pub name: String,
    /// 电镀
    pub plating: String,
    /// 色号/颜色
    pub color: String,
    /// 颜色
    pub color_2: Option<String>,
    /// 尺寸
    pub size: Option<String>,
    /// 条码
    pub barcode: Option<String>,
    /// 数量
    pub count: i32,
    /// 进货价
    pub purchase_price: Option<i32>,
    /// 单位
    pub unit: Option<String>,
    /// 单价
    pub unit_price: Option<i32>,
    /// 金额
    pub total_price: Option<i32>,
    /// 备注
    pub notes: Option<String>,
}

impl OrderItemExcel {
    pub fn to_order_item_no_id_model(&self, order_id: i32, sku_id: i32) -> OrderItemNoIdModel {
        OrderItemNoIdModel {
            order_id,
            sku_id,
            package_card: self.package_card.clone(),
            package_card_des: self.package_card_des.clone(),
            count: self.count,
            unit: self.unit.clone(),
            unit_price: self.unit_price,
            total_price: self.total_price,
            notes: self.notes.clone(),
        }
    }

    pub fn pick_up_goods_no(items: &Vec<OrderItemExcel>) -> Option<String> {
        let mut goods_no_cnt: HashMap<&str, i32> = HashMap::new();

        let _ = items.iter().map(|item| {
            *goods_no_cnt.entry(&item.goods_no).or_default() + 1;
        });

        let key = key_of_max_value(&goods_no_cnt).unwrap_or(&"").to_string();
        let empty_string = "".to_string();
        if key.is_empty() {
            // 如果找不到goods_no,怎从sku_no里获取(最大的prefix)
            let sku_nos = items
                .iter()
                .map(|item| item.sku_no.as_ref().unwrap_or(&empty_string).clone())
                .collect::<Vec<_>>();

            return common_prefix(sku_nos);
        }

        Some(key)
    }

    pub fn pick_up_goods(items: &Vec<OrderItemExcel>) -> GoodsModel {
        let mut goods = GoodsModel {
            id: 0,
            goods_no: "".to_string(),
            image: "".to_string(),
            name: "".to_string(),
            notes: None,
        };

        goods.goods_no = OrderItemExcel::pick_up_goods_no(&items).unwrap();
        for item in items {
            if goods.image.is_empty() && item.image.is_some() {
                goods.image = item.image.as_ref().unwrap().to_string();
            }
            if goods.name.is_empty() && !item.name.is_empty() {
                goods.name = item.name.clone();
            }
        }

        goods
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItemMaterialModel {
    pub id: i32,
    pub order_id: i32,
    pub order_item_id: i32,
    pub name: String,
    pub color: String,
    // material_id   integer, -- 材料ID  (暂时先不用)
    pub single: Option<i32>,   //  integer, -- 单数      ？小数
    pub count: i32,            //  integer, -- 数量      ？小数
    pub total: Option<i32>,    //  integer, -- 总数(米)  ? 小数
    pub stock: Option<i32>,    //  integer, -- 库存 ?
    pub debt: Option<i32>,     //  integer, -- 欠数
    pub notes: Option<String>, //  text,     -- 备注
}
