use crate::constants::{STORAGE_FILE_PATH, STORAGE_URL_PREFIX};
use crate::model::order::OrderItemExcel;
use umya_spreadsheet::*;
use crate::common::string::remove_whitespace_str;

pub fn parse_order_excel_t3(sheet: &Worksheet) -> Vec<OrderItemExcel> {
    let (cols, rows) = sheet.get_highest_column_and_row();
    let mut items = vec![];

    let mut pre: Option<OrderItemExcel> = None;
    for i in 7..rows + 1 {
        let mut cur = OrderItemExcel::default();
        if let Some(previous) = pre.as_ref() {
            cur = previous.clone();
        }
        let mut goods_image: Option<Image> = None;

        for j in 1..cols + 1 {
            if j == 2 {
                if let Some(real_image) = sheet.get_image((j, i)) {
                    goods_image = Some(real_image.clone());
                }
            }

            let cell = sheet.get_cell((j, i));
            if cell.is_none() {
                continue;
            }

            let cell_value = cell.unwrap().get_raw_value().to_string();
            if cell_value.is_empty() {
                continue;
            }

            match j {
                1 => cur.index = cell_value.parse::<i32>().unwrap_or(0),
                3 => cur.goods_no = remove_whitespace_str(&cell_value),
                4 => cur.name = cell_value,
                5 => cur.plating = cell_value,
                6 => cur.color = remove_whitespace_str(&cell_value),
                7 => cur.color_2 = Some(cell_value),
                8 => cur.barcode = Some(cell_value),
                9 => cur.purchase_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                10 => cur.count = cell_value.parse::<i32>().unwrap_or(0),
                11 => cur.total_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                12 => cur.notes = Some(cell_value),
                _ => {}
            }
        }

        if let Some(real_goods_image) = goods_image {
            let goods_image_path = format!("{}/sku/{}.png", STORAGE_FILE_PATH, cur.goods_no);
            real_goods_image.download_image(&goods_image_path);
            cur.image = Some(format!("{}/sku/{}.png", STORAGE_URL_PREFIX, cur.goods_no));
        }

        items.push(cur.clone());
        pre = Some(cur);
    }

    items
}

#[cfg(test)]
mod tests {
    use crate::excel::order_template_3::parse_order_excel_t3;
    use umya_spreadsheet::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let path =
            std::path::Path::new("/Users/ligangzhou/Money/rust/erp-api/excel_templates/L1012.xlsx");
        let book = reader::xlsx::read(path)?;
        let sheet = book.get_active_sheet();
        let order_info = parse_order_excel_t3(sheet);
        println!("order_info: {:#?}", order_info);
        Ok(())
    }
}
