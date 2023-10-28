use crate::common::hashmap::key_of_max_value;
use crate::common::string::common_prefix;
use crate::model::goods::{GoodsModel, SKUModel};
use crate::{ERPError, ERPResult};
use chrono::NaiveDate;
use sqlx::{FromRow, Pool, Postgres, QueryBuilder};
use std::collections::HashMap;

#[derive(Debug, Serialize, Clone, FromRow)]
pub struct OrderModel {
    pub id: i32,
    pub customer_no: String,
    pub order_no: String,
    pub order_date: NaiveDate,
    pub delivery_date: Option<NaiveDate>,
    pub is_urgent: bool,          //紧急 ‼️
    pub is_return_order: bool,    // 返单
    pub is_special: bool,         // 特别客人
    pub special_customer: String, //特别客人
}

impl OrderModel {
    pub async fn get_order_with_order_no(
        db: &Pool<Postgres>,
        order_no: &str,
    ) -> ERPResult<Option<OrderModel>> {
        let order = sqlx::query_as!(
            OrderModel,
            "select * from orders where order_no=$1",
            order_no
        )
        .fetch_optional(db)
        .await
        .map_err(ERPError::DBError)?;

        Ok(order)
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderGoodsModel {
    pub id: i32,
    pub index: i32,
    pub images: Vec<String>,
    pub image_des: String,
    pub package_card: String,
    pub package_card_des: String,
    pub order_id: i32,
    pub goods_id: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Default, Clone)]
pub struct GoodsImagesAndPackageModel {
    pub images: Vec<String>,
    pub image_des: String,
    pub package_card: String,
    pub package_card_des: String,
    pub goods_id: i32,
}

impl OrderGoodsModel {
    pub async fn get_goods_images_and_package(
        db: &Pool<Postgres>,
        goods_id: i32,
    ) -> ERPResult<GoodsImagesAndPackageModel> {
        let goods_images_package = sqlx::query_as!(
            GoodsImagesAndPackageModel,
            r#"
            select goods_id, images, image_des, package_card, package_card_des 
            from order_goods 
            where goods_id=$1 
            order by id desc 
            limit 1
            "#,
            goods_id
        )
        .fetch_optional(db)
        .await
        .map_err(ERPError::DBError)?
        .ok_or(ERPError::NotFound("有商品未找到".to_string()))?;

        Ok(goods_images_package)
    }

    pub async fn get_multiple_goods_images_and_package(
        db: &Pool<Postgres>,
        goods_ids: &[i32],
    ) -> ERPResult<HashMap<i32, GoodsImagesAndPackageModel>> {
        let goods_images_package_hash = sqlx::query_as!(
            GoodsImagesAndPackageModel,
            r#"
            select distinct on (goods_id)
            goods_id, images, image_des, package_card, package_card_des
            from order_goods
            where goods_id = any($1)
            order by goods_id desc, id desc;
            "#,
            goods_ids
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?
        .into_iter()
        .map(|images_package| (images_package.goods_id, images_package))
        .collect::<HashMap<i32, GoodsImagesAndPackageModel>>();

        Ok(goods_images_package_hash)
    }
}

impl OrderGoodsModel {
    pub async fn add_rows(
        db: &Pool<Postgres>,
        rows: &[OrderGoodsModel],
    ) -> ERPResult<Vec<OrderGoodsModel>> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("insert into order_goods (index, order_id, goods_id, images, image_des, package_card, package_card_des) ");

        query_builder.push_values(rows, |mut b, item| {
            b.push_bind(item.index)
                .push_bind(item.order_id)
                .push_bind(item.goods_id)
                .push_bind(item.images.clone())
                .push_bind(item.image_des.clone())
                .push_bind(item.package_card.clone())
                .push_bind(item.package_card_des.clone());
        });
        query_builder.push(" returning *;");

        let res = query_builder
            .build_query_as::<OrderGoodsModel>()
            .fetch_all(db)
            .await
            .map_err(ERPError::DBError)?;

        Ok(res)
    }

    pub async fn get_row(
        db: &Pool<Postgres>,
        order_id: i32,
        goods_id: i32,
    ) -> ERPResult<Option<OrderGoodsModel>> {
        let row = sqlx::query_as!(
            OrderGoodsModel,
            "select * from order_goods where order_id=$1 and goods_id=$2;",
            order_id,
            goods_id
        )
        .fetch_optional(db)
        .await
        .map_err(ERPError::DBError)?;

        Ok(row)
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItemModel {
    pub id: i32,
    pub order_goods_id: i32, // todo: 应该是比存goods_id好
    pub order_id: i32,
    pub sku_id: i32,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes_images: Vec<String>,
    pub notes: String,
}

impl OrderItemModel {
    pub async fn save_to_order_item_table(
        db: &Pool<Postgres>,
        items: &[OrderItemModel],
    ) -> ERPResult<Vec<OrderItemModel>> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("insert into order_items (order_goods_id, order_id, sku_id, count, unit, unit_price, total_price, notes_images, notes) ");

        query_builder.push_values(items, |mut b, item| {
            b.push_bind(item.order_goods_id)
                .push_bind(item.order_id)
                .push_bind(item.sku_id)
                .push_bind(item.count)
                .push_bind(item.unit.as_deref().unwrap_or(""))
                .push_bind(item.unit_price.unwrap_or(0))
                .push_bind(item.total_price.unwrap_or(0))
                .push_bind(item.notes_images.clone())
                .push_bind(item.notes.clone());
        });
        query_builder.push(" returning *;");

        let res = query_builder
            .build_query_as::<OrderItemModel>()
            .fetch_all(db)
            .await
            .map_err(ERPError::DBError)?;

        Ok(res)
    }
    pub async fn get_order_items_with_order_goods_id(
        db: &Pool<Postgres>,
        order_goods_id: i32,
    ) -> ERPResult<Vec<OrderItemModel>> {
        let order_items = sqlx::query_as!(
            OrderItemModel,
            "select * from order_items where order_goods_id=$1",
            order_goods_id
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?;

        Ok(order_items)
    }

    pub async fn get_rows_with_order_id_and_goods_id(
        db: &Pool<Postgres>,
        order_id: i32,
        goods_id: i32,
    ) -> ERPResult<Vec<OrderItemModel>> {
        let items = sqlx::query_as!(
            OrderItemModel,
            r#"
            select oi.*
            from order_items oi, order_goods og
            where oi.order_goods_id = og.id
                and oi.order_id = $1 and og.goods_id=$2
            "#,
            order_id,
            goods_id
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?;

        Ok(items)
    }
}

#[derive(Debug, Clone)]
pub struct ExcelOrderGoods {
    pub index: i32,
    pub goods_no: String,
    pub images: Vec<String>,
    pub image_des: Option<String>,
    pub name: String,
    pub plating: String,
    pub package_card: Option<String>,
    pub package_card_des: Option<String>,
}

impl ExcelOrderGoods {
    pub async fn insert_into_goods_table(
        db: &Pool<Postgres>,
        items: &[ExcelOrderGoods],
        customer_no: &str,
    ) -> ERPResult<HashMap<String, i32>> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("insert into goods (customer_no, goods_no, images, image_des, name, plating, package_card, package_card_des) ");

        query_builder.push_values(items, |mut b, item| {
            b.push_bind(customer_no)
                .push_bind(item.goods_no.clone())
                .push_bind(item.images.clone())
                .push_bind(item.image_des.as_deref().unwrap_or(""))
                .push_bind(item.name.clone())
                .push_bind(item.plating.clone())
                .push_bind(item.package_card.as_deref().unwrap_or(""))
                .push_bind(item.package_card_des.as_deref().unwrap_or(""));
        });
        query_builder.push(" returning goods_no, id;");

        let res = query_builder
            .build_query_as::<(String, i32)>()
            .fetch_all(db)
            .await
            .map_err(ERPError::DBError)?
            .into_iter()
            .map(|item| (item.0, item.1))
            .collect::<HashMap<String, i32>>();

        Ok(res)
    }

    pub async fn insert_into_skus_table(
        db: &Pool<Postgres>,
        items: &[SKUModel],
    ) -> ERPResult<Vec<SKUModel>> {
        let mut query_builder: QueryBuilder<Postgres> =
            QueryBuilder::new("insert into skus (goods_id, sku_no, color, color2) ");

        query_builder.push_values(items, |mut b, item| {
            b.push_bind(item.goods_id)
                .push_bind(&item.sku_no)
                .push_bind(&item.color)
                .push_bind(&item.color2);
        });
        query_builder.push(" returning *;");

        let res = query_builder
            .build_query_as::<SKUModel>()
            .fetch_all(db)
            .await
            .map_err(ERPError::DBError)?;

        Ok(res)
    }
}

#[derive(Debug, Clone)]
pub struct ExcelOrderItems {
    /// 颜色
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

#[derive(Debug, Clone)]
pub struct ExcelOrderGoodsWithItems {
    pub goods: ExcelOrderGoods,
    pub items: Vec<OrderItemExcel>,
}

#[derive(Debug, Clone)]
pub struct ExcelOrderV2 {
    pub info: OrderInfo,
    pub items: Vec<ExcelOrderGoodsWithItems>,
    pub exists: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ExcelOrder {
    pub info: OrderInfo,
    pub items: Vec<OrderItemExcel>,
    pub exists: bool,
}

#[derive(Default, Debug, Clone)]
pub struct OrderInfo {
    pub customer_no: String,
    pub order_no: String,
    pub order_date: NaiveDate,
    pub delivery_date: Option<NaiveDate>,
    pub is_return_order: bool,
    pub is_urgent: bool,
    pub is_special: bool,
    pub special_customer: String,
}

impl OrderInfo {
    pub async fn insert_to_orders(db: &Pool<Postgres>, order_info: &OrderInfo) -> ERPResult<i32> {
        let order_id = sqlx::query!(
            r#"
            insert into orders (customer_no, order_no, order_date, delivery_date, is_urgent, is_return_order)
            values ($1, $2, $3, $4, $5, $6) returning id;
            "#,
            order_info.customer_no,
            order_info.order_no,
            order_info.order_date,
            order_info.delivery_date,
            order_info.is_urgent,
            order_info.is_return_order
        ).fetch_one(db).await
            .map_err(|_| ERPError::Failed("插入订单失败".to_string()))?
            .id;

        Ok(order_id)
    }

    pub async fn update_to_orders(
        db: &Pool<Postgres>,
        order_info: &OrderInfo,
        order_id: i32,
    ) -> ERPResult<()> {
        sqlx::query!(
            r#"
            update orders set customer_no=$1, order_no=$2, order_date=$3, delivery_date=$4, is_urgent=$5, is_return_order=$6
            where id = $7
            "#,
            order_info.customer_no,
            order_info.order_no,
            order_info.order_date,
            order_info.delivery_date,
            order_info.is_urgent,
            order_info.is_return_order,
            order_id
        ).execute(db).await.map_err(|_| ERPError::Failed("覆盖订单信息失败".to_string()))?;

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
    // /// 商品编号
    // pub goods_no_2: Option<String>, // 反正用处并不大
    /// sku编号 //只有L1005有这个字段
    pub sku_no: Option<String>,
    /// 商品图片
    pub images: Vec<String>,
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
    /// 备注里的图片列表
    pub notes_images: Vec<String>,
    /// 备注
    pub notes: Option<String>,
}

impl OrderItemExcel {
    pub fn pick_up_goods_no(items: &[OrderItemExcel]) -> Option<String> {
        let mut goods_no_cnt: HashMap<&str, i32> = HashMap::new();

        items.iter().for_each(|item| {
            if !item.goods_no.is_empty() {
                *goods_no_cnt.entry(&item.goods_no).or_insert(0) += 1;
            }
        });
        tracing::info!("goods_no_cnt: {:?}", goods_no_cnt);

        let key = key_of_max_value(&goods_no_cnt).unwrap_or(&"").to_string();
        tracing::info!("goods_no_cnt key: {}", key);
        let empty_string = "".to_string();
        if key.is_empty() {
            // 如果找不到goods_no,怎从sku_no里获取(最大的prefix)
            let sku_nos = items
                .iter()
                .map(|item| item.sku_no.as_ref().unwrap_or(&empty_string).clone())
                .collect::<Vec<_>>();

            tracing::info!("sku_nos: {:?}", sku_nos);
            return common_prefix(sku_nos);
        }

        Some(key)
    }

    pub fn pick_up_excel_goods(items: &Vec<OrderItemExcel>) -> ExcelOrderGoods {
        let mut goods = ExcelOrderGoods {
            index: 0,
            goods_no: "".to_string(),
            images: vec![],
            image_des: None,
            name: "".to_string(),
            plating: "".to_string(),
            package_card: None,
            package_card_des: None,
        };

        goods.goods_no = OrderItemExcel::pick_up_goods_no(items).unwrap();
        items.iter().for_each(|item| {
            if goods.index == 0 && item.index > 0 {
                goods.index = item.index;
            }
            if goods.images.is_empty() && !item.images.is_empty() {
                goods.images = item.images.clone();
            }
            if goods.image_des.is_none() && item.image_des.is_some() {
                goods.image_des = item.image_des.clone();
            }
            if goods.name.is_empty() && !item.name.is_empty() {
                goods.name = item.name.clone();
            }
            if goods.plating.is_empty() && !item.plating.is_empty() {
                goods.plating = item.plating.clone();
            }
            if goods.package_card.is_none() && item.package_card.is_some() {
                goods.package_card = item.package_card.clone();
            }
            if goods.package_card_des.is_none() && item.package_card_des.is_some() {
                goods.package_card_des = item.package_card_des.clone();
            }
        });

        goods
    }

    // pub fn pick_up_goods(items: &Vec<OrderItemExcel>) -> GoodsModel {
    //     let mut goods = GoodsModel {
    //         id: 0,
    //         customer_no: "".to_string(),
    //         goods_no: "".to_string(),
    //         // images: vec![],
    //         // image_des: "".to_string(),
    //         name: "".to_string(),
    //         plating: "".to_string(),
    //         // package_card: "".to_string(),
    //         // package_card_des: "".to_string(),
    //         notes: "".to_string(),
    //     };
    //
    //     goods.goods_no = OrderItemExcel::pick_up_goods_no(items).unwrap();
    //     items.iter().for_each(|item| {
    //         // if goods.images.is_empty() && !item.images.is_empty() {
    //         //     goods.images = item.images.clone();
    //         // }
    //         if goods.name.is_empty() && !item.name.is_empty() {
    //             goods.name = item.name.clone();
    //         }
    //         if goods.plating.is_empty() && !item.plating.is_empty() {
    //             goods.plating = item.plating.clone();
    //         }
    //     });
    //
    //     goods
    // }

    pub fn pick_up_package(items: &Vec<OrderItemExcel>) -> (String, String) {
        let mut package_card: Option<String> = None;
        let mut package_card_des: Option<String> = None;

        items.iter().for_each(|item| {
            if package_card.is_none() && item.package_card.is_some() {
                package_card = item.package_card.clone();
            }
            if package_card_des.is_none() && item.package_card_des.is_some() {
                package_card_des = item.package_card_des.clone();
            }
        });

        (
            package_card.unwrap_or("".to_string()),
            package_card_des.unwrap_or("".to_string()),
        )
    }

    pub async fn save_to_sku(&self, db: &Pool<Postgres>, goods_id: i32) -> ERPResult<i32> {
        let id = sqlx::query!(
            r#"
            insert into skus (goods_id, sku_no, color, color2)
            values ($1, $2, $3, $4)
            returning id;
            "#,
            goods_id,
            self.sku_no.as_deref().unwrap_or(""),
            self.color,
            self.color_2.as_deref().unwrap_or("")
        )
        .fetch_one(db)
        .await
        .map_err(ERPError::DBError)?
        .id;
        Ok(id)
    }

    pub async fn save_to_order_item(
        &self,
        db: &Pool<Postgres>,
        order_id: i32,
        order_goods_id: i32,
        sku_id: i32,
    ) -> ERPResult<i32> {
        let id = sqlx::query!(
            r#"
            insert into order_items (order_id, order_goods_id, sku_id, count, unit, unit_price, total_price, notes)
            values ($1, $2, $3, $4, $5, $6, $7, $8)
            returning id;"#,
            order_id,
            order_goods_id,
            sku_id,
            self.count,
            self.unit.as_deref().unwrap_or(""),
            self.unit_price.as_ref().unwrap_or(&0),
            self.total_price.as_ref().unwrap_or(&0),
            self.notes.as_deref().unwrap_or("")
        )
        .fetch_one(db)
        .await
        .map_err(ERPError::DBError)?
            .id;

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
