use crate::excel::excel_order_info::parse_order_info;
use crate::excel::order_template_1::{checking_order_items_excel_1, parse_order_excel_t1};
use crate::excel::order_template_2::{checking_order_items_excel_2, parse_order_excel_t2};
use crate::excel::order_template_3::{checking_order_items_excel_3, parse_order_excel_t3};
use crate::excel::order_template_4::{checking_order_items_excel_4, parse_order_excel_t4};
use crate::model::customer::CustomerModel;
use crate::model::excel::CustomerExcelTemplateModel;
use crate::model::goods::{GoodsModel, SKUModel};
use crate::model::order::{
    ExcelOrder, OrderGoodsModel, OrderInfo, OrderItemExcel, OrderItemModel, OrderModel,
};
use crate::{ERPError, ERPResult};
use itertools::Itertools;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use umya_spreadsheet::reader;

#[derive(Debug)]
pub struct ExcelOrderParser<'a> {
    pub path: &'a str,
    db: Pool<Postgres>,
}

impl<'a> ExcelOrderParser<'a> {
    pub fn new(path: &'a str, db: Pool<Postgres>) -> ExcelOrderParser<'a> {
        Self { path, db }
    }

    pub async fn parse(&self) -> ERPResult<ExcelOrder> {
        // parse order_info
        let path = std::path::Path::new(self.path);
        let book = reader::xlsx::read(path)
            .map_err(|_| ERPError::Failed("读xlsx文件失败,不支持xls格式".to_owned()))?;
        let sheet = book.get_active_sheet();
        let order_info = parse_order_info(sheet);

        // find which template is for this customer.
        if order_info.customer_no.is_empty() {
            return Err(ERPError::Failed(
                "客户编号未找到，请检查一下excel表格".to_string(),
            ));
        }
        if order_info.order_no.is_empty() {
            return Err(ERPError::Failed(
                "订单编号未找到，请检查一下excel表格".to_string(),
            ));
        }

        // let customer = sqlx::query_as::<_, CustomerModel>(&format!(
        //     "select * from customers where customer_no='{}'",
        //     order_info.customer_no
        // ))
        // .fetch_one(&self.db)
        // .await
        // .map_err(|_| ERPError::NotFound(format!("客户#{}未找到", &order_info.customer_no)))?;

        // let customer_id = customer.id;

        tracing::info!("customer_no: {}", &order_info.customer_no);
        let customer_excel_template_model = sqlx::query_as!(
            CustomerExcelTemplateModel,
            "select * from customer_excel_template where customer_no=$1",
            &order_info.customer_no
        )
        .fetch_optional(&self.db)
        .await
        .map_err(ERPError::DBError)?;

        if customer_excel_template_model.is_none() {
            return Err(ERPError::Failed(format!(
                "请先配置{}需要使用什么模版",
                &order_info.customer_no
            )));
        }

        let template_id = customer_excel_template_model.unwrap().template_id;

        let order_items = match template_id {
            1 => parse_order_excel_t1(sheet),
            2 => parse_order_excel_t2(sheet),
            3 => parse_order_excel_t3(sheet),
            4 => parse_order_excel_t4(sheet),
            _ => parse_order_excel_t1(sheet),
        };

        match template_id {
            1 => checking_order_items_excel_1(&order_items)?,
            2 => checking_order_items_excel_2(&order_items)?,
            3 => checking_order_items_excel_3(&order_items)?,
            4 => checking_order_items_excel_4(&order_items)?,
            _ => checking_order_items_excel_1(&order_items)?,
        };

        let mut excel_order = ExcelOrder {
            info: order_info,
            items: order_items,
            exists: false,
        };

        tracing::info!("excel_order: {:?}", excel_order);

        // 判断order_no是否已经存在
        let order =
            OrderModel::get_order_with_order_no(&self.db, &excel_order.info.order_no).await?;

        tracing::info!("order: {:?}", order);
        let customer =
            CustomerModel::get_customer_with_customer_no(&self.db, &excel_order.info.customer_no)
                .await?;

        // 订单是否已经存在
        // 如果已经存在，则更新一下，如果不存在则 保存
        let order_id = {
            match order {
                None => {
                    tracing::info!(
                        "order#{} not exists, we will save",
                        excel_order.info.order_no
                    );

                    OrderInfo::insert_to_orders(&self.db, &excel_order.info).await?
                }
                Some(existing_order) => {
                    tracing::info!("订单#{}已存在,尝试更新数据", excel_order.info.order_no);
                    excel_order.exists = true;
                    OrderInfo::update_to_orders(
                        &self.db,
                        &excel_order.info,
                        customer.id,
                        existing_order.id,
                    )
                    .await?;
                    existing_order.id
                }
            }
        };

        // check goods/skus exists.
        let mut id_order_item: HashMap<i32, Vec<OrderItemExcel>> = HashMap::new();
        for item in excel_order.items.iter() {
            id_order_item
                .entry(item.index)
                .or_insert(vec![])
                .push(item.clone())
        }
        tracing::info!("id_order_items: {:?}", id_order_item);

        // #[derive(Debug)]
        // struct ExistingOrderItem {}
        // 先获取当前所有的商品+skus
        // let existing = sqlx::query_as::<_, (i32, )>(&format!("select id from order_goods where "))
        // TODO: 循环检查 商品是否已经入库
        for (index, items) in id_order_item.iter().sorted_by_key(|x| x.0) {
            let goods_no = OrderItemExcel::pick_up_goods_no(items).unwrap();
            tracing::info!("picked up goods_no: {}", goods_no);

            let goods = GoodsModel::get_goods_with_goods_no(&self.db, &goods_no).await?;
            tracing::info!("goods: {:?}", goods);

            let goods_id = match goods {
                None => {
                    let goods = OrderItemExcel::pick_up_goods(items);
                    GoodsModel::insert_goods_to_db(&self.db, &goods, &excel_order.info.customer_no)
                        .await?
                }
                Some(some_goods) => some_goods.id,
            };
            tracing::info!("goods_id: {goods_id}");

            // 处理order_goods
            let order_goods = OrderGoodsModel::get_row(&self.db, order_id, goods_id).await?;
            tracing::info!("order_goods: {:?}", order_goods);
            if order_goods.is_none() {
                let (package_card, package_card_des) = OrderItemExcel::pick_up_package(&items);
                tracing::info!("package: {package_card}, {package_card_des}");

                // insert order_goods data
                sqlx::query!(
                    r#"insert into order_goods(index, order_id, goods_id)
                    values ($1, $2, $3);"#,
                    index,
                    order_id,
                    goods_id
                )
                .execute(&self.db)
                .await
                .map_err(ERPError::DBError)?;
            }

            // 处理items
            let skus = SKUModel::get_skus_with_goods_id(&self.db, goods_id).await?;
            let mut color_to_id = skus
                .iter()
                .map(|sku| (sku.color.clone(), sku.id))
                .collect::<HashMap<String, i32>>();

            // 处理order_items
            let order_items =
                OrderItemModel::get_rows_with_order_id_and_goods_id(&self.db, order_id, goods_id)
                    .await?;

            let sku_id_to_order_item_id = order_items
                .iter()
                .map(|item| (item.sku_id, item.id))
                .collect::<HashMap<i32, i32>>();

            for item in items.iter() {
                let sku_id = match color_to_id.get(&item.color) {
                    None => {
                        // insert to table items
                        let id = item.save_to_sku(&self.db, goods_id).await?;
                        color_to_id.insert(item.color.to_owned(), id);
                        id
                    }
                    Some(sku_id) => sku_id.to_owned(),
                };
                if sku_id_to_order_item_id.contains_key(&sku_id) {
                    // 更新数据
                    // todo,感觉可以不做
                } else {
                    // 插入数据
                    let order_item_id = item
                        .save_to_order_item(&self.db, order_id, goods_id, sku_id)
                        .await?;
                    tracing::info!("save to order_items#{order_item_id}");
                }
            }
        }

        Ok(excel_order)
    }
}
