use crate::common::string::remove_whitespace_str;
use crate::constants::{STORAGE_FILE_PATH, STORAGE_URL_PREFIX};
use crate::model::order::OrderItemExcel;
use crate::{ERPError, ERPResult};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;
use umya_spreadsheet::*;

pub fn parse_order_excel_t1(sheet: &Worksheet) -> Vec<OrderItemExcel> {
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

pub fn checking_order_items_excel_1(order_items_excel: &Vec<OrderItemExcel>) -> ERPResult<()> {
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
                "Excel内序号#{index}可能重复,或者有多余总计的行"
            )));
        }
    }

    Ok(())
}

#[derive(Clone, Debug, Default)]
struct GoodsInfo<'a> {
    pub goods_no: &'a str,
    pub image: &'a str,
    pub name: &'a str,
    pub plating: &'a str,
    pub package_card: &'a str,
    pub package_card_des: &'a str,
    pub notes: &'a str,
}

pub async fn get_order_no_to_order_id<'a>(
    db: &Pool<Postgres>,
    order_items_excel: &'a Vec<OrderItemExcel>,
) -> ERPResult<HashMap<&'a str, i32>> {
    let mut goods_no_to_goods_info: HashMap<&'a str, GoodsInfo> = HashMap::new();

    for item in order_items_excel {
        let goods_info = goods_no_to_goods_info
            .entry(&item.goods_no)
            .or_insert(GoodsInfo::default());
    }

    let mut order_nos = order_items_excel
        .iter()
        .map(|item| item.goods_no.as_str())
        .collect::<Vec<&'a str>>();

    if order_nos.is_empty() {
        return Ok(HashMap::new());
    }

    order_nos.dedup();
    let order_nos_str = order_nos.join(",");

    let existing_order_nos = sqlx::query_as::<_, (i32, String)>(&format!(
        "select id, goods_no from goods where goods_no in ({})",
        order_nos_str
    ))
    .fetch_all(db)
    .await
    .map_err(ERPError::DBError)?;

    todo!()
}

#[cfg(test)]
mod tests {
    use crate::excel::order_template_1::parse_order_excel_t1;
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
