use crate::excel::excel_order_info::parse_order_info;
use crate::excel::parse_order_template_1::parse_order_excel_t1;
use crate::excel::parse_order_template_2::parse_order_excel_t2;
use crate::excel::parse_order_template_3::parse_order_excel_t3;
use crate::excel::parse_order_template_4::parse_order_excel_t4;
use crate::excel::process_order_excel_goods::{
    convert_index_vec_order_item_excel_to_vec_excel_order_goods_with_items,
    process_order_excel_with_goods_no_and_sku_color,
};
use crate::model::excel::CustomerExcelTemplateModel;
use crate::model::order::{ExcelOrderV2, OrderInfo, OrderModel};
use crate::{ERPError, ERPResult};
use sqlx::{Pool, Postgres};
use umya_spreadsheet::reader;

#[derive(Debug)]
pub struct ExcelOrderParser<'a> {
    pub path: &'a str,
    db: Pool<Postgres>,
    build_by: i32,
}

impl<'a> ExcelOrderParser<'a> {
    pub fn new(path: &'a str, db: Pool<Postgres>, build_by: i32) -> ExcelOrderParser<'a> {
        Self { path, db, build_by }
    }

    pub async fn parse(&self) -> ERPResult<ExcelOrderV2> {
        // parse order_info
        let path = std::path::Path::new(self.path);
        let book = reader::xlsx::read(path)
            .map_err(|_| ERPError::Failed("读xlsx文件失败,不支持xls格式".to_string()))?;
        let sheet = book.get_sheet(&0).unwrap();
        let order_info = parse_order_info(sheet)?;

        tracing::info!("order_info: {:?}", order_info);
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

        // find which template is for this customer.
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
            1 => parse_order_excel_t1(sheet, &order_info.order_no)?,
            2 => parse_order_excel_t2(sheet, &order_info.order_no)?,
            3 => parse_order_excel_t3(sheet, &order_info.order_no)?,
            4 => parse_order_excel_t4(sheet, &order_info.order_no)?,
            _ => parse_order_excel_t1(sheet, &order_info.order_no)?,
        };

        println!("order_items: {:?}", order_items);
        let no_goods_no = matches!(template_id, 2);
        let order_goods_item =
            convert_index_vec_order_item_excel_to_vec_excel_order_goods_with_items(
                order_items,
                no_goods_no,
            )?;

        // 判断order_no是否已经存在
        let mut order_exists = false;
        let order = OrderModel::get_order_with_order_no(&self.db, &order_info.order_no).await?;

        let order_id = {
            match order {
                None => {
                    tracing::info!("order#{} not exists, we will save", &order_info.order_no);
                    OrderInfo::insert_to_orders(&self.db, &order_info, self.build_by).await?
                }
                Some(existing_order) => {
                    tracing::info!("订单#{}已存在,尝试更新数据", &order_info.order_no);
                    order_exists = true;
                    OrderInfo::update_to_orders(
                        &self.db,
                        &order_info,
                        self.build_by,
                        existing_order.id,
                    )
                    .await?;
                    existing_order.id
                }
            }
        };

        match template_id {
            1 => {
                process_order_excel_with_goods_no_and_sku_color(
                    &self.db,
                    &order_goods_item,
                    &order_info,
                    order_id,
                )
                .await?
            }
            _ => {
                process_order_excel_with_goods_no_and_sku_color(
                    &self.db,
                    &order_goods_item,
                    &order_info,
                    order_id,
                )
                .await?
            }
        };

        let excel_order = ExcelOrderV2 {
            info: order_info,
            items: order_goods_item,
            exists: order_exists,
        };

        Ok(excel_order)
    }
}
