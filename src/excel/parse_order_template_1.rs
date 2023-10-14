use crate::common::string::remove_whitespace_str;
use crate::constants::{STORAGE_FILE_PATH, STORAGE_URL_PREFIX};
use crate::model::order::OrderItemExcel;
use crate::{ERPError, ERPResult};
use std::collections::HashMap;
use umya_spreadsheet::*;

pub fn parse_order_excel_t1(sheet: &Worksheet) -> ERPResult<HashMap<i32, Vec<OrderItemExcel>>> {
    let (cols, rows) = sheet.get_highest_column_and_row();

    // 先获得了 HashMap<index, vec<Row>>
    let mut index_to_items = HashMap::new();
    let mut pre: Option<OrderItemExcel> = None;

    for i in 7..rows + 1 {
        let mut cur = OrderItemExcel::default();
        if let Some(previous) = pre {
            cur = previous.clone();
        }

        let mut package_image: Option<Image> = None;
        let mut goods_images: Vec<&Image> = vec![];

        for j in 1..cols + 1 {
            if j == 2 {
                if let Some(real_image) = sheet.get_image((j, i)) {
                    package_image = Some(real_image.clone());
                }
            }
            if j == 4 {
                goods_images = sheet.get_images((j, i));
            }

            let cell = sheet.get_cell((j, i));
            if cell.is_none() {
                // if j == 1 {
                //     // 如果是第一格是空的，就当作是空行/
                //     return Err(ERPError::ExcelError(format!(
                //         "第{i}行可能有空行，因为没有读到index的数据"
                //     )));
                // }
                continue;
            }

            let cell_value = cell.unwrap().get_raw_value().to_string();
            if cell_value.is_empty() {
                // if j == 1 {
                //     // 如果是第一格是空的，就当作是空行/
                //     return Err(ERPError::ExcelError(format!(
                //         "第{i}行可能有空行，因为没有读到index的数据"
                //     )));
                // }
                continue;
            }

            match j {
                1 => cur.index = cell_value.parse::<i32>().unwrap_or(0),
                2 => cur.package_card_des = Some(cell_value.trim().to_string()),
                3 => cur.goods_no = remove_whitespace_str(&cell_value),
                4 => cur.image_des = Some(cell_value.trim().to_string()),
                5 => cur.name = cell_value.trim().to_string(),
                6 => cur.plating = cell_value.trim().to_string(),
                7 => cur.color = remove_whitespace_str(&cell_value),
                8 => cur.count = cell_value.parse::<i32>().unwrap_or(0),
                9 => cur.unit = Some(cell_value.trim().to_string()),
                10 => cur.unit_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                11 => cur.total_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                12 => cur.notes = Some(cell_value.trim().to_string()),
                _ => {}
            }
        }

        let mut image_urls = vec![];
        if !goods_images.is_empty() {
            for (index, real_goods_image) in goods_images.into_iter().enumerate() {
                let goods_image_path =
                    format!("{}/sku/{}-{}.png", STORAGE_FILE_PATH, cur.goods_no, index);
                real_goods_image.download_image(&goods_image_path);
                image_urls.push(format!(
                    "{}/sku/{}-{}.png",
                    STORAGE_URL_PREFIX, cur.goods_no, index
                ))
            }
        }
        cur.images = image_urls;

        if let Some(read_package_image) = package_image {
            let package_image_path = format!("{}/package/{}.png", STORAGE_FILE_PATH, cur.goods_no);
            read_package_image.download_image(&package_image_path);
            cur.package_card = Some(format!(
                "{}/package/{}.png",
                STORAGE_URL_PREFIX, cur.goods_no
            ));
        }

        if cur.index == 0 {
            return Err(ERPError::ExcelError(format!(
                "第{i}行可能有空行，因为没有读到index的数据"
            )));
        }

        index_to_items
            .entry(cur.index)
            .or_insert(vec![])
            .push(cur.clone());
        pre = Some(cur);
    }

    Ok(index_to_items)
}

#[cfg(test)]
mod tests {
    use crate::excel::parse_order_template_1::parse_order_excel_t1;
    use umya_spreadsheet::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let path =
            std::path::Path::new("/Users/ligangzhou/Money/rust/erp-api/excel_templates/L1001.xlsx");
        let book = reader::xlsx::read(path)?;
        let sheet = book.get_active_sheet();
        let order_info = parse_order_excel_t1(sheet);
        tracing::info!("order_info: {:#?}", order_info);
        Ok(())
    }
}
