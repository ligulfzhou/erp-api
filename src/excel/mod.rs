use umya_spreadsheet::Worksheet;

pub mod excel_order_info;
pub mod excel_order_parser;
pub mod order_template_1;
pub mod order_template_2;


pub trait OrderExcelParser {
    fn parse_order_info(&self, sheet: &Worksheet);
    fn parse_order_items(&self, sheet: &Worksheet);
}