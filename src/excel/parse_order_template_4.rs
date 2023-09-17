use crate::common::string::remove_whitespace_str;
use crate::constants::{STORAGE_FILE_PATH, STORAGE_URL_PREFIX};
use crate::model::order::OrderItemExcel;
use crate::{ERPError, ERPResult};
use std::collections::HashMap;
use umya_spreadsheet::*;

pub fn parse_order_excel_t4(sheet: &Worksheet) -> ERPResult<HashMap<i32, Vec<OrderItemExcel>>> {
    let (cols, rows) = sheet.get_highest_column_and_row();

    let mut index_to_items = HashMap::new();
    let mut pre: Option<OrderItemExcel> = None;
    for i in 7..rows + 1 {
        let mut cur = OrderItemExcel::default();
        if let Some(previous) = pre.as_ref() {
            cur = previous.clone();
        }
        let mut package_image: Option<Image> = None;
        let mut goods_image: Option<Image> = None;

        for j in 1..cols + 1 {
            if j == 2 || j == 7 {
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
                if j == 1 {
                    // 如果是第一格是空的，就当作是空行/
                    return Err(ERPError::ExcelError(format!(
                        "第{i}行可能有空行，因为没有读到index的数据"
                    )));
                }
                continue;
            }

            let cell_value = cell.unwrap().get_raw_value().to_string();
            if cell_value.is_empty() {
                if j == 1 {
                    // 如果是第一格是空的，就当作是空行/
                    return Err(ERPError::ExcelError(format!(
                        "第{i}行可能有空行，因为没有读到index的数据"
                    )));
                }
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

        if let Some(real_goods_image) = goods_image {
            let goods_image_path = format!("{}/sku/{}.png", STORAGE_FILE_PATH, identifier);
            real_goods_image.download_image(&goods_image_path);
            cur.image = Some(format!("{}/sku/{}.png", STORAGE_URL_PREFIX, identifier));
        }

        if let Some(read_package_image) = package_image {
            let package_image_path = format!("{}/package/{}.png", STORAGE_FILE_PATH, identifier);
            read_package_image.download_image(&package_image_path);
            cur.package_card = Some(format!("{}/package/{}.png", STORAGE_URL_PREFIX, identifier));
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

pub fn checking_order_items_excel_4(order_items_excel: &[OrderItemExcel]) -> ERPResult<()> {
    let mut index_order_items = HashMap::new();

    for order_item in order_items_excel.iter() {
        index_order_items
            .entry(order_item.index)
            .or_insert(vec![])
            .push(order_item);
    }

    for (index, order_items) in index_order_items.iter() {
        let mut goods_nos = order_items
            .iter()
            .map(|item| item.goods_no.as_str())
            .collect::<Vec<&str>>();
        goods_nos.dedup();
        if goods_nos.len() > 1 {
            return Err(ERPError::ExcelError(format!(
                "Excel内序号#{index}可能重复,或者有 多余总计的行"
            )));
        }
    }

    for order_item in order_items_excel.iter() {
        if order_item.color.is_empty() {
            return Err(ERPError::ExcelError(format!(
                "序列#{}可能有重复，或者有多余的总计的行",
                order_item.index
            )));
        }
    }

    Ok(())
}

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
