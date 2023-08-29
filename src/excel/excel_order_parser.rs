use crate::excel::excel_order_info::parse_order_info;
use crate::excel::order_template_1::parse_order_excel_t1;
use crate::excel::order_template_2::parse_order_excel_t2;
use crate::excel::order_template_3::parse_order_excel_t3;
use crate::excel::order_template_4::parse_order_excel_t4;
use crate::model::excel::CustomerExcelTemplateModel;
use crate::model::order::{ExcelOrder, OrderModel};
use crate::{ERPError, ERPResult};
use sqlx::{Pool, Postgres};
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

        // 订单是否已经存在
        // 如果已经存在，则更新一下，如果不存在则 保存
        match order {
            None => {
                // save order
                tracing::info!(
                    "order#{} not exists, we will save",
                    excel_order.info.order_no
                );
                let (customer_id,) = sqlx::query_as::<_, (i32,)>(&format!(
                    "select id from customers where customer_no='{}'",
                    excel_order.info.customer_no
                ))
                .fetch_one(&self.db)
                .await
                .map_err(ERPError::DBError)?;

                let delivery_date = match excel_order.info.delivery_date {
                    None => "null".to_string(),
                    Some(dt) => dt.format("'%Y-%m-%d'").to_string(),
                };

                let insert_sql = format!(
                    r#"insert into orders (customer_id, order_no, order_date, delivery_date, is_urgent, is_return_order)
                    values ({}, '{}', '{:?}', {}, {}, {}) returning id;"#,
                    customer_id,
                    excel_order.info.order_no,
                    excel_order.info.order_date,
                    delivery_date,
                    excel_order.info.is_urgent,
                    excel_order.info.is_return_order
                );
                tracing::info!("insert order sql: {}", insert_sql);
                let (order_id,) = sqlx::query_as::<_, (i32,)>(&insert_sql)
                    .fetch_one(&self.db)
                    .await
                    .map_err(ERPError::DBError)?; // &self.db)

                tracing::info!("order_id: {}", order_id);
            }
            Some(existing_order) => {
                // maybe update order
            }
        }

        Ok(excel_order)
    }
}
