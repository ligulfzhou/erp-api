use umya_spreadsheet::*;

#[derive(Debug, Default, Clone)]
pub struct OrderItemExcel {
    pub index: i32,
    pub package_card: Option<String>,
    pub package_card_des: Option<String>,
    pub customer_order_no: Option<String>,
    pub image: Option<String>,
    pub image_des: Option<String>,
    pub goods_no: String,
    pub name: String,
    pub plating: String,
    pub color: String,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes: Option<String>,
}

pub fn read_excel_with_umya(file_path: &str) -> Vec<OrderItemExcel> {
    let path = std::path::Path::new(file_path);
    let mut book = reader::xlsx::read(path).unwrap();

    let sheet = book.get_active_sheet();
    let (cols, rows) = sheet.get_highest_column_and_row();
    // println!("{:?}",sheet.get_highest_row());
    // println!("{:?}", sheet.get_highest_column());
    // println!("{:?}", sheet.get_highest_column_and_row());
    //
    // println!("get (2,7)");
    // let cell = sheet.get_cell((2, 7));
    // if cell.is_some() {
    //     println!("(2,7) raw value: {:?}", cell.unwrap().get_raw_value());
    // }
    // let images = sheet.get_images((2, 7));
    // println!("(2,7) images: {:?}", images.len());
    // let images = sheet.get_images((2, 8));
    // println!("(2,7) images: {:?}", images.len());
    let mut items = vec![];

    let mut pre: Option<OrderItemExcel> = None;
    for i in 7..rows + 1 {
        let mut cur = OrderItemExcel::default();
        if let Some(previous) = pre.as_ref() {
            cur = previous.clone();
        }
        // if pre.is_some() {
        //     cur = pre.unwrap().clone()
        // }
        let mut package_image: Option<Image> = None;
        let mut goods_image: Option<Image> = None;

        for j in 1..cols + 1 {
            if j == 2 || j == 4 {
                let image = sheet.get_image((j, i));
                if image.is_some() {
                    if j == 2 {
                        package_image = Some(image.unwrap().clone());
                    } else {
                        goods_image = Some(image.unwrap().clone());
                    }
                }
            }

            let cell = sheet.get_cell((j, i));
            if !cell.is_some() {
                continue;
            }

            let cell_value = cell.unwrap().get_raw_value().to_string();
            if cell_value.is_empty() {
                continue;
            }

            match j {
                1 => cur.index = cell_value.parse::<i32>().unwrap_or(0),
                2 => cur.package_card_des = Some(cell_value),
                3 => cur.customer_order_no = Some(cell_value),
                4 => cur.image_des = Some(cell_value),
                5 => cur.goods_no = cell_value,
                6 => cur.name = cell_value,
                7 => cur.plating = cell_value,
                8 => cur.color = cell_value,
                9 => cur.count = cell_value.parse::<i32>().unwrap_or(0),
                10 => cur.unit = Some(cell_value),
                11 => cur.unit_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                12 => cur.total_price = Some(cell_value.parse::<i32>().unwrap_or(0)),
                13 => cur.notes = Some(cell_value),
                _ => {}
            }
        }

        if let Some(real_goods_image) = goods_image {
            real_goods_image.download_image(&format!("./{}.png", cur.goods_no));
        }
        if let Some(read_package_image) = package_image {
            read_package_image.download_image(&format!("./package_{}.png", cur.goods_no));
        }
        items.push(cur.clone());
        pre = Some(cur);
    }

    for (index, item) in items.iter().enumerate() {
        println!("{index}: {:?}", item);
    }

    items
}

#[cfg(test)]
mod tests {
    use crate::excel::order_umya_excel::read_excel_with_umya;

    #[test]
    fn test() {
        read_excel_with_umya("/xyz");
    }
}
