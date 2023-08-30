use crate::common::hashmap::key_of_max_value;
use crate::common::string::common_prefix;
use crate::model::goods::GoodsModel;
use crate::{ERPError, ERPResult};
use chrono::NaiveDate;
use sqlx::{Pool, Postgres};
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
impl OrderModel {
    pub async fn get_order_with_order_no(
        db: &Pool<Postgres>,
        order_no: &str,
    ) -> ERPResult<Option<OrderModel>> {
        let order = sqlx::query_as::<_, OrderModel>(&format!(
            "select * from orders where order_no='{}'",
            order_no
        ))
        .fetch_optional(db)
        .await
        .map_err(ERPError::DBError)?;

        Ok(order)
    }
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

impl OrderGoodsModel {
    pub async fn get_row(
        db: &Pool<Postgres>,
        order_id: i32,
        goods_id: i32,
    ) -> ERPResult<Option<OrderGoodsModel>> {
        let sql = format!(
            "select * from order_goods where order_id={} and goods_id={};",
            order_id, goods_id
        );
        let row = sqlx::query_as::<_, OrderGoodsModel>(&sql)
            .fetch_optional(db)
            .await
            .map_err(ERPError::DBError)?;
        Ok(row)
    }
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
    // pub purchase_price: Option<i32>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes: Option<String>,
}

impl OrderItemModel {
    pub async fn get_rows_with_order_id_and_goods_id(
        db: &Pool<Postgres>,
        order_id: i32,
        goods_id: i32,
    ) -> ERPResult<Vec<OrderItemModel>> {
        let sql = format!(
            "select * from order_items where order_id={} and goods_id={}",
            order_id, goods_id
        );
        let items = sqlx::query_as::<_, OrderItemModel>(&sql)
            .fetch_all(db)
            .await
            .map_err(ERPError::DBError)?;
        Ok(items)
    }
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

impl OrderInfo {
    pub async fn insert_to_orders(
        db: &Pool<Postgres>,
        order_info: &OrderInfo,
        customer_id: i32,
    ) -> ERPResult<i32> {
        let delivery_date = match order_info.delivery_date {
            None => "null".to_string(),
            Some(dt) => dt.format("'%Y-%m-%d'").to_string(),
        };

        let insert_sql = format!(
            r#"insert into orders (customer_id, order_no, order_date, delivery_date, is_urgent, is_return_order)
                    values ({}, '{}', '{:?}', {}, {}, {}) returning id;"#,
            customer_id,
            order_info.order_no,
            order_info.order_date,
            delivery_date,
            order_info.is_urgent,
            order_info.is_return_order
        );
        tracing::info!("insert order sql: {}", insert_sql);
        let (order_id,) = sqlx::query_as::<_, (i32,)>(&insert_sql)
            .fetch_one(db)
            .await
            .map_err(ERPError::DBError)?;

        Ok(order_id)
    }

    pub async fn update_to_orders(
        db: &Pool<Postgres>,
        order_info: &OrderInfo,
        customer_id: i32,
        order_id: i32,
    ) -> ERPResult<()> {
        let delivery_date = match order_info.delivery_date {
            None => "null".to_string(),
            Some(dt) => dt.format("'%Y-%m-%d'").to_string(),
        };

        let update_sql = format!(
            r#"update orders set customer_id={}, order_no='{}', order_date='{:?}', delivery_date={}, is_urgent={}, is_return_order={}
               where id = {};"#,
            customer_id,
            order_info.order_no,
            order_info.order_date,
            delivery_date,
            order_info.is_urgent,
            order_info.is_return_order,
            order_id
        );
        tracing::info!("update sql: {}", update_sql);

        sqlx::query(&update_sql)
            .execute(db)
            .await
            .map_err(ERPError::DBError)?;
        Ok(())
    }
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
    pub fn pick_up_goods_no(items: &Vec<OrderItemExcel>) -> Option<String> {
        let mut goods_no_cnt: HashMap<&str, i32> = HashMap::new();

        for item in items.iter() {
            // *goods_no_cnt.entry(&item.goods_no).or_default() + 1;
            let count = goods_no_cnt.entry(&item.goods_no).or_insert(0);
            *count += 1;
        }
        println!("goods_no_cnt: {:?}", goods_no_cnt);

        let key = key_of_max_value(&goods_no_cnt).unwrap_or(&"").to_string();
        println!("goods_no_cnt key: {}", key);
        let empty_string = "".to_string();
        if key.is_empty() {
            // 如果找不到goods_no,怎从sku_no里获取(最大的prefix)
            let sku_nos = items
                .iter()
                .map(|item| item.sku_no.as_ref().unwrap_or(&empty_string).clone())
                .collect::<Vec<_>>();

            println!("sku_nos: {:?}", sku_nos);
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

    pub fn pick_up_package(items: &Vec<OrderItemExcel>) -> (String, String) {
        let mut package_card: Option<String> = None;
        let mut package_card_des: Option<String> = None;

        for item in items {
            if package_card.is_none() && item.package_card.is_some() {
                package_card = item.package_card.clone();
            }
            if package_card_des.is_none() && !item.package_card_des.is_some() {
                package_card_des = item.package_card_des.clone();
            }
        }

        (
            package_card.unwrap_or("".to_string()),
            package_card_des.unwrap_or("".to_string()),
        )
    }

    pub async fn save_to_sku(&self, db: &Pool<Postgres>, goods_id: i32) -> ERPResult<i32> {
        let sql = format!(
            r#"insert into skus (goods_id, plating, sku_no, color, color2)
            values ({}, '{}', '{}', '{}', '{}')
            returning id;"#,
            goods_id,
            self.plating,
            self.sku_no.as_deref().unwrap_or(""),
            self.color,
            self.color_2.as_deref().unwrap_or("")
        );

        let (sku_id,) = sqlx::query_as::<_, (i32,)>(&sql)
            .fetch_one(db)
            .await
            .map_err(ERPError::DBError)?;

        Ok(sku_id)
    }

    pub async fn save_to_order_item(
        &self,
        db: &Pool<Postgres>,
        order_id: i32,
        goods_id: i32,
        sku_id: i32,
    ) -> ERPResult<i32> {
        let sql = format!(
            r#"
            insert into order_items (order_id, goods_id, sku_id, count, unit, unit_price, total_price, notes)
            values ({}, {}, {}, {}, '{}', {}, {}, '{}')
            returning id;
        "#,
            order_id,
            goods_id,
            sku_id,
            self.count,
            self.unit.as_deref().unwrap_or(""),
            self.unit_price.as_ref().unwrap_or(&0),
            self.total_price.as_ref().unwrap_or(&0),
            self.notes.as_deref().unwrap_or("")
        );
        let (id,) = sqlx::query_as::<_, (i32,)>(&sql)
            .fetch_one(db)
            .await
            .map_err(ERPError::DBError)?;
        Ok(id)
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
