use crate::common::string::{common_prefix, remove_whitespace_str};
use crate::constants::{STORAGE_FILE_PATH, STORAGE_URL_PREFIX};
use crate::model::order::OrderItemExcel;
use crate::{ERPError, ERPResult};
use std::collections::HashMap;
use umya_spreadsheet::*;

pub fn parse_order_excel_t2(sheet: &Worksheet) -> Vec<OrderItemExcel> {
    let (cols, rows) = sheet.get_highest_column_and_row();
    tracing::info!("cols: {cols}, rows: {rows}");

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

        items.push(cur.clone());
        pre = Some(cur);
    }

    items
}

pub fn checking_order_items_excel_2(order_items_excel: &[OrderItemExcel]) -> ERPResult<()> {
    let sku_nos = order_items_excel
        .iter()
        .map(|item| item.sku_no.as_deref().unwrap_or(""))
        .collect::<Vec<&str>>();

    let mut sku_nos_clone = sku_nos.clone();
    sku_nos_clone.dedup();
    if sku_nos_clone.len() != sku_nos.len() {
        let mut sku_no_to_count = HashMap::new();
        for sku_no in sku_nos {
            *sku_no_to_count.entry(sku_no).or_insert(0) += 1;
        }

        let dup_sku_nos = sku_no_to_count
            .iter()
            .filter_map(|item| {
                if item.1 > &1 {
                    Some(item.0.to_owned())
                } else {
                    None
                }
            })
            .collect::<Vec<&str>>()
            .join(",");

        return Err(ERPError::ExcelError(format!(
            "编号#{dup_sku_nos}可能重复,或者有多余总计的行"
        )));
    }

    let mut index_order_items = HashMap::new();

    for order_item in order_items_excel.iter() {
        index_order_items
            .entry(order_item.index)
            .or_insert(vec![])
            .push(order_item);
    }

    for (index, order_items) in index_order_items.iter() {
        let sku_nos = order_items
            .iter()
            .map(|item| item.sku_no.as_deref().unwrap_or("").to_string())
            .collect::<Vec<String>>();

        let goods_no = common_prefix(sku_nos);
        if goods_no.is_none() || goods_no.unwrap().len() != 6 {
            return Err(ERPError::ExcelError(format!(
                "序号#{index}的编号可能有错误"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::excel::parse_order_template_2::parse_order_excel_t2;
    use umya_spreadsheet::*;

    #[test]
    fn test() -> anyhow::Result<()> {
        let path =
            std::path::Path::new("/Users/ligangzhou/Money/rust/erp-api/excel_templates/L1005.xlsx");
        let book = reader::xlsx::read(path)?;
        let sheet = book.get_active_sheet();
        let order_info = parse_order_excel_t2(sheet);
        tracing::info!("order_info: {:#?}", order_info);

        Ok(())
    }
}