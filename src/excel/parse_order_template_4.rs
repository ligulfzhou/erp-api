use crate::common::string::remove_whitespace_str;
use crate::constants::{STORAGE_FILE_PATH, STORAGE_URL_PREFIX};
use crate::model::order::OrderItemExcel;
use crate::{ERPError, ERPResult};
use std::collections::HashMap;
use umya_spreadsheet::*;

pub fn parse_order_excel_t4(
    sheet: &Worksheet,
    order_no: &str,
) -> ERPResult<HashMap<i32, Vec<OrderItemExcel>>> {
    let (cols, rows) = sheet.get_highest_column_and_row();

    let mut index_to_items = HashMap::new();
    let mut pre: Option<OrderItemExcel> = None;
    for i in 7..rows + 1 {
        let mut cur = OrderItemExcel::default();
        if let Some(previous) = pre.as_ref() {
            cur = previous.clone();
            cur.notes_images = vec![];
        }
        let mut package_image: Option<Image> = None;
        let mut goods_images: Vec<&Image> = vec![];
        let mut notes_images: Vec<&Image> = vec![];

        for j in 1..cols + 1 {
            if j == 2 {
                if let Some(real_image) = sheet.get_image((j, i)) {
                    package_image = Some(real_image.clone());
                }
            }
            if j == 7 {
                goods_images = sheet.get_images((j, i));
            }
            if j == 15 {
                notes_images = sheet.get_images((j, i));
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
                3 => cur.sku_no = Some(remove_whitespace_str(&cell_value)),
                4 => cur.goods_no = remove_whitespace_str(&cell_value),
                5 => cur.color = remove_whitespace_str(&cell_value),
                6 => cur.size = Some(cell_value.trim().to_string()),
                7 => cur.image_des = Some(cell_value.trim().to_string()),
                8 => cur.name = cell_value.trim().to_string(),
                9 => cur.plating = cell_value.trim().to_string(),
                10 => cur.color_2 = Some(cell_value.trim().to_string()),
                11 => cur.count = cell_value.parse::<i32>().unwrap_or(0),
                12 => cur.unit = Some(cell_value.trim().to_string()),
                13 => cur.unit_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                14 => cur.total_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                15 => cur.notes = Some(cell_value.trim().to_string()),
                _ => {}
            }
        }

        let mut identifier = cur.goods_no.clone();
        if identifier.is_empty() {
            identifier = cur.sku_no.as_ref().unwrap().clone();
        }

        let mut image_urls = vec![];
        if !goods_images.is_empty() {
            for (index, real_goods_image) in goods_images.into_iter().enumerate() {
                let sku_image_name = format!("{}-{}-{}.png", cur.goods_no, index, order_no);
                let goods_image_path = format!("{}/sku/{}", STORAGE_FILE_PATH, sku_image_name);
                real_goods_image.download_image(&goods_image_path);
                image_urls.push(format!("{}/sku/{}", STORAGE_URL_PREFIX, sku_image_name));
            }
        }
        cur.images = image_urls;

        if let Some(read_package_image) = package_image {
            let package_image_name = format!("{}-{}.png", cur.goods_no, order_no);
            let package_image_path =
                format!("{}/package/{}", STORAGE_FILE_PATH, package_image_name);
            read_package_image.download_image(&package_image_path);
            cur.package_card = Some(format!(
                "{}/package/{}",
                STORAGE_URL_PREFIX, package_image_name
            ));
        }

        let mut notes_image_urls = vec![];
        if !notes_images.is_empty() {
            for (index, real_notes_image) in notes_images.into_iter().enumerate() {
                let notes_image_name = format!("{}-{}-{}.png", cur.goods_no, index, order_no);
                let notes_image_path = format!("{}/notes/{}", STORAGE_FILE_PATH, notes_image_name);
                real_notes_image.download_image(&notes_image_path);
                notes_image_urls.push(format!("{}/notes/{}", STORAGE_URL_PREFIX, notes_image_name));
            }
        }
        cur.notes_images = notes_image_urls;

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

// see: convert_index_vec_order_item_excel_to_vec_excel_order_goods_with_items
// pub fn checking_order_items_excel_4(order_items_excel: &[OrderItemExcel]) -> ERPResult<()> {
//     let mut index_order_items = HashMap::new();
//
//     for order_item in order_items_excel.iter() {
//         index_order_items
//             .entry(order_item.index)
//             .or_insert(vec![])
//             .push(order_item);
//     }
//
//     for (index, order_items) in index_order_items.iter() {
//         let mut goods_nos = order_items
//             .iter()
//             .map(|item| item.goods_no.as_str())
//             .collect::<Vec<&str>>();
//         goods_nos.dedup();
//         if goods_nos.len() > 1 {
//             return Err(ERPError::ExcelError(format!(
//                 "Excel内序号#{index}可能重复,或者有 多余总计的行"
//             )));
//         }
//     }
//
//     for order_item in order_items_excel.iter() {
//         if order_item.color.is_empty() {
//             return Err(ERPError::ExcelError(format!(
//                 "序列#{}可能有重复，或者有多余的总计的行",
//                 order_item.index
//             )));
//         }
//     }
//
//     Ok(())
// }

#[cfg(test)]
mod tests {
    use crate::excel::parse_order_template_4::parse_order_excel_t4;
    use umya_spreadsheet::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let path =
            std::path::Path::new("/Users/ligangzhou/Money/rust/erp-api/excel_templates/L1004.xlsx");
        let book = reader::xlsx::read(path)?;
        let sheet = book.get_active_sheet();
        let order_info = parse_order_excel_t4(sheet);
        tracing::info!("order_info: {:#?}", order_info);

        // order_info.iter().map(|item|tracing::info!("{:?}", item));
        Ok(())
    }
}
