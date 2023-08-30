use crate::common::string::remove_whitespace_str;
use crate::constants::{STORAGE_FILE_PATH, STORAGE_URL_PREFIX};
use crate::model::order::OrderItemExcel;
use umya_spreadsheet::*;

pub fn parse_order_excel_t2(sheet: &Worksheet) -> Vec<OrderItemExcel> {
    let (cols, rows) = sheet.get_highest_column_and_row();
    let mut items = vec![];

    let mut pre: Option<OrderItemExcel> = None;
    for i in 7..rows + 1 {
        let mut cur = OrderItemExcel::default();
        if let Some(previous) = pre.as_ref() {
            cur = previous.clone();
        }
        let mut package_image: Option<Image> = None;
        let mut goods_image: Option<Image> = None;

        for j in 1..cols + 1 {
            if j == 2 || j == 4 {
                if let Some(real_image) = sheet.get_image((j, i)) {
                    if j == 2 {
                        package_image = Some(real_image.clone());
                    } else {
                        goods_image = Some(real_image.clone());
                    }
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
                2 => cur.package_card_des = Some(cell_value.trim().to_string()),
                3 => cur.sku_no = Some(remove_whitespace_str(&cell_value)),
                4 => cur.image_des = Some(cell_value.trim().to_string()),
                5 => cur.name = cell_value.trim().to_string(),
                6 => cur.plating = cell_value.trim().to_string(),
                7 => cur.color = cell_value.trim().to_string(),
                8 => cur.count = cell_value.parse::<i32>().unwrap_or(0),
                9 => cur.unit = Some(cell_value.trim().to_string()),
                10 => cur.unit_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                11 => cur.total_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                12 => cur.notes = Some(cell_value.trim().to_string()),
                _ => {}
            }
        }

        if let Some(real_goods_image) = goods_image {
            let goods_image_path = format!("{}/sku/{}.png", STORAGE_FILE_PATH, cur.goods_no);
            real_goods_image.download_image(&goods_image_path);
            cur.image = Some(format!("{}/sku/{}.png", STORAGE_URL_PREFIX, cur.goods_no));
        }

        if let Some(read_package_image) = package_image {
            let package_image_path = format!("{}/package/{}.png", STORAGE_FILE_PATH, cur.goods_no);
            read_package_image.download_image(&package_image_path);
            cur.package_card = Some(format!(
                "{}/package/{}.png",
                STORAGE_URL_PREFIX, cur.goods_no
            ));
        }

        items.push(cur.clone());
        pre = Some(cur);
    }

    items
}

#[cfg(test)]
mod tests {
    use crate::excel::order_template_2::parse_order_excel_t2;
    use umya_spreadsheet::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let path =
            std::path::Path::new("/Users/ligangzhou/Money/rust/erp-api/excel_templates/L1005.xlsx");
        let book = reader::xlsx::read(path)?;
        let sheet = book.get_active_sheet();
        let order_info = parse_order_excel_t2(sheet);
        println!("order_info: {:#?}", order_info);

        // order_info.iter().map(|item|tracing::info!("{:?}", item));
        Ok(())
    }
}
