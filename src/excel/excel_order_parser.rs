use crate::excel::excel_order_info::parse_order_info;
use crate::excel::order_template_1::parse_order_excel_t1;
use crate::model::excel::CustomerExcelTemplateModel;
use crate::model::order::ExcelOrder;
use crate::{ERPError, ERPResult};
use sqlx::{Pool, Postgres};
use std::fs::read;
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

        let order_items = match customer_excel_template_model.unwrap().template_id {
            1 => parse_order_excel_t1(self.path),
            2 => parse_order_excel_t1(self.path),
            3 => parse_order_excel_t1(self.path),
            _ => parse_order_excel_t1(self.path),
        };

        let excel_order = ExcelOrder {
            info: order_info,
            items: order_items,
        };

        Ok(excel_order)
    }
}
