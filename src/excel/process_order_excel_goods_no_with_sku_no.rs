use crate::error::ERPResult;
use crate::model::goods::SKUModel;
use crate::model::order::{ExcelOrderGoods, ExcelOrderGoodsWithItems, OrderInfo};
use crate::ERPError;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

pub async fn process_order_excel_with_goods_no_and_sku_no(
    db: &Pool<Postgres>,
    order_goods_excel: &Vec<ExcelOrderGoodsWithItems>,
    order_info: &OrderInfo,
) -> ERPResult<()> {
    let goods_nos = order_goods_excel
        .iter()
        .map(|item| item.goods.goods_no.to_string())
        .collect::<Vec<String>>();

    let mut existing_goods_no_to_id = sqlx::query!(
        "select id, goods_no from goods where goods_no = any($1)",
        &goods_nos
    )
    .fetch_all(db)
    .await
    .map_err(ERPError::DBError)?
    .into_iter()
    .map(|item| (item.goods_no, item.id))
    .collect::<HashMap<String, i32>>();
    println!("existing_id_goods_no: {:?}", existing_goods_no_to_id);

    let to_add_goods_nos = goods_nos
        .iter()
        .filter(|item| !existing_goods_no_to_id.contains_key(*item))
        .map(|item| item.as_str())
        .collect::<Vec<&str>>();

    println!(
        "to_add_goods_nos: {:?}, len: {}",
        to_add_goods_nos,
        to_add_goods_nos.len()
    );

    if !to_add_goods_nos.is_empty() {
        let to_add_goods = order_goods_excel
            .iter()
            .filter(|item| to_add_goods_nos.contains(&item.goods.goods_no.as_str()))
            .map(|item| item.goods.clone())
            .collect::<Vec<ExcelOrderGoods>>();

        println!(
            "to_add_goods: {:?}, len: {}",
            to_add_goods,
            to_add_goods.len()
        );

        let new_goods_no_to_id =
            ExcelOrderGoods::insert_into_goods_table(db, &to_add_goods, &order_info.customer_no)
                .await?;

        println!("new_goods_no_to_id: {:?}", new_goods_no_to_id);

        for (goods_no, id) in new_goods_no_to_id.into_iter() {
            existing_goods_no_to_id.insert(goods_no, id);
        }
    }
    println!("goods_no_to_id: {:?}", existing_goods_no_to_id);

    // 用goods_ids去获取所有的skus，如果数据没有入库，则入库
    let goods_ids = existing_goods_no_to_id
        .iter()
        .map(|item| *item.1)
        .collect::<Vec<i32>>();

    let skus = sqlx::query_as!(
        SKUModel,
        "select * from skus where goods_id = any($1)",
        &goods_ids
    )
    .fetch_all(db)
    .await
    .map_err(ERPError::DBError)?;

    let mut sku_no_to_id = skus
        .iter()
        .map(|item| (item.sku_no.as_str(), item.id))
        .collect::<HashMap<&str, i32>>();
    let sku_nos = skus
        .iter()
        .map(|item| item.sku_no.as_str())
        .collect::<Vec<&str>>();

    let mut skus_to_add = vec![];
    for order_goods in order_goods_excel {
        let goods_id = existing_goods_no_to_id
            .get(&order_goods.goods.goods_no)
            .unwrap_or(&0);
        for order_goods_sku in order_goods.items.iter() {
            let sku_no = order_goods_sku.sku_no.as_deref().unwrap_or("");
            if !sku_nos.contains(&sku_no) {
                skus_to_add.push(SKUModel {
                    id: 0,
                    goods_id: *goods_id,
                    sku_no: sku_no.to_string(),
                    color: order_goods_sku.color.to_string(),
                    color2: order_goods_sku.color_2.as_deref().unwrap_or("").to_string(),
                    notes: None,
                })
            }
        }
    }

    if !skus_to_add.is_empty() {
        let new_sku_no_to_id = ExcelOrderGoods::insert_into_skus_table(db, &skus_to_add).await?;
        for (sku_no, id) in new_sku_no_to_id.iter() {
            sku_no_to_id.insert(sku_no.as_str(), *id);
        }
    }

    Ok(())
}
