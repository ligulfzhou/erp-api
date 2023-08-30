use crate::excel::excel_order_info::parse_order_info;
use crate::excel::order_template_1::parse_order_excel_t1;
use crate::excel::order_template_2::parse_order_excel_t2;
use crate::excel::order_template_3::parse_order_excel_t3;
use crate::excel::order_template_4::parse_order_excel_t4;
use crate::model::excel::CustomerExcelTemplateModel;
use crate::model::goods::GoodsModel;
use crate::model::order::{ExcelOrder, OrderInfo, OrderItemExcel, OrderModel};
use crate::{ERPError, ERPResult};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use umya_spreadsheet::reader;
use crate::model::customer::CustomerModel;

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
        let book =
            reader::xlsx::read(path).map_err(|_| ERPError::Failed("读xlsx文件失败".to_owned()))?;
        let sheet = book.get_active_sheet();
        let order_info = parse_order_info(sheet);

        // find which template is for this customer.
        if order_info.customer_no.is_empty() {
            return Err(ERPError::Failed(
                "客户编号未找到，请检查一下excel表格".to_string(),
            ));
        }

        tracing::info!("customer_no: {}", order_info.customer_no);
        let customer_excel_template_model =
            sqlx::query_as::<_, CustomerExcelTemplateModel>(&format!(
                "select * from customer_excel_template where customer_no='{}';",
                order_info.customer_no
            ))
            .fetch_optional(&self.db)
            .await
            .map_err(ERPError::DBError)?;

        if customer_excel_template_model.is_none() {
            return Err(ERPError::Failed(format!(
                "请先配置{}需要使用什么模版",
                order_info.customer_no
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

        let excel_order = ExcelOrder {
            info: order_info,
            items: order_items,
        };

        tracing::info!("excel_order: {:?}", excel_order);

        // 判断order_no是否已经存在
        let order = sqlx::query_as::<_, OrderModel>(&format!(
            "select * from orders where order_no='{}'",
            excel_order.info.order_no
        ))
        .fetch_optional(&self.db)
        .await
        .map_err(ERPError::DBError)?;

        let customer = CustomerModel::get_customer_with_customer_no(&self.db, &excel_order.info.customer_no).await?;
        let mut order_id = 0;

        // 订单是否已经存在
        // 如果已经存在，则更新一下，如果不存在则 保存
        match order {
            None => {
                // save order
                tracing::info!(
                    "order#{} not exists, we will save",
                    excel_order.info.order_no
                );

                order_id =
                    OrderInfo::insert_to_orders(&self.db, &excel_order.info, customer.id).await?;
                tracing::info!("order_id: {}", order_id);
            }
            Some(existing_order) => {
                // maybe update order
                tracing::info!("订单#{}已存在,尝试更新数据", excel_order.info.order_no);
                OrderInfo::update_to_orders(
                    &self.db,
                    &excel_order.info,
                    customer.id,
                    existing_order.id,
                )
                .await?;
                order_id = existing_order.id;
            }
        }

        // check goods/skus exists.
        let mut id_order_item: HashMap<i32, Vec<OrderItemExcel>> = HashMap::new();
        for item in excel_order.items.iter() {
            id_order_item.entry(item.index).or_insert(vec![]).push(item.clone())
        }

        tracing::info!("id_order_items: {:?}", id_order_item);

        // 循环检查 商品是否已经入库
        for (index, items) in id_order_item.iter() {
            let goods_no = OrderItemExcel::pick_up_goods_no(items).unwrap();
            let goods = GoodsModel::get_goods_with_goods_no(&self.db, &goods_no)
                .await
                .unwrap();

            let goods_id = match goods {
                None => {
                    // insert goods
                    let goods = OrderItemExcel::pick_up_goods(items);
                    GoodsModel::insert_goods_to_db(&self.db, &goods)
                        .await
                        .unwrap()
                }
                Some(some_goods) => some_goods.id,
            };

        }

        // check order_items
        Ok(excel_order)
    }
}
