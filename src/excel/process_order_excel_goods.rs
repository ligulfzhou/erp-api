use crate::error::ERPResult;
use crate::model::goods::SKUModel;
use crate::model::order::{
    ExcelOrderGoods, ExcelOrderGoodsWithItems, OrderGoodsModel, OrderInfo, OrderItemModel,
};
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
        .map(|item| item.goods.goods_no.clone())
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
        let new_skus = ExcelOrderGoods::insert_into_skus_table(db, &skus_to_add).await?;
        for new_sku in new_skus.iter() {
            sku_no_to_id.insert(new_sku.sku_no.as_str(), new_sku.id);
        }
    }

    Ok(())
}

pub async fn process_order_excel_with_goods_no_and_sku_color(
    db: &Pool<Postgres>,
    order_goods_excel: &Vec<ExcelOrderGoodsWithItems>,
    order_info: &OrderInfo,
    order_id: i32,
) -> ERPResult<()> {
    // 首先是查看goods_no有没有入库
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
        print!("order_goods_excel: {:?}", order_goods_excel);
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

    let mut goods_id_to_color_to_sku_id = HashMap::new();
    for sku in skus.into_iter() {
        goods_id_to_color_to_sku_id
            .entry(sku.goods_id)
            .or_insert(HashMap::new())
            .insert(sku.color, sku.id);
    }

    let mut skus_to_add = vec![];
    let empty_hashmap: HashMap<String, i32> = HashMap::new();
    for order_goods in order_goods_excel {
        let goods_id = existing_goods_no_to_id
            .get(&order_goods.goods.goods_no)
            .unwrap_or(&0);

        println!("goods_id: {}", goods_id);

        for order_goods_sku in order_goods.items.iter() {
            if !goods_id_to_color_to_sku_id
                .get(goods_id)
                .unwrap_or(&empty_hashmap)
                .contains_key(&order_goods_sku.color)
            {
                skus_to_add.push(SKUModel {
                    id: 0,
                    goods_id: *goods_id,
                    sku_no: "".to_string(),
                    color: order_goods_sku.color.to_string(),
                    color2: order_goods_sku.color_2.as_deref().unwrap_or("").to_string(),
                    notes: None,
                })
            }
        }
    }

    println!("skus_to_add: {:?}", skus_to_add);
    if !skus_to_add.is_empty() {
        let new_skus = ExcelOrderGoods::insert_into_skus_table(db, &skus_to_add).await?;
        for new_sku in new_skus.into_iter() {
            goods_id_to_color_to_sku_id
                .entry(new_sku.goods_id)
                .or_insert(HashMap::new())
                .insert(new_sku.color, new_sku.id);
        }
    }

    // 添加 order_goods
    let existing_order_goods = sqlx::query_as!(
        OrderGoodsModel,
        "select * from order_goods where order_id=$1 and goods_id=any($2)",
        order_id,
        &goods_ids
    )
    .fetch_all(db)
    .await
    .map_err(ERPError::DBError)?;

    let mut goods_id_to_order_goods_id = existing_order_goods
        .iter()
        .map(|item| (item.goods_id, item.id))
        .collect::<HashMap<i32, i32>>();

    if existing_order_goods.len() < goods_ids.len() {
        // 添加order_goods
        let mut to_add_order_goods = vec![];
        for order_goods in order_goods_excel {
            let this_goods_id = existing_goods_no_to_id
                .get(&order_goods.goods.goods_no)
                .unwrap_or(&0);
            if !goods_id_to_order_goods_id.contains_key(this_goods_id) {
                to_add_order_goods.push(OrderGoodsModel {
                    id: 0,
                    index: order_goods.goods.index,
                    order_id,
                    goods_id: *this_goods_id,
                })
            }
        }
        if !to_add_order_goods.is_empty() {
            let new_order_goods = OrderGoodsModel::add_rows(db, &to_add_order_goods).await?;
            for new_order_good in new_order_goods.into_iter() {
                goods_id_to_order_goods_id.insert(new_order_good.goods_id, new_order_good.id);
            }
        }
    }

    // 添加 order_items
    let order_goods_ids = goods_id_to_order_goods_id
        .iter()
        .map(|item| *item.1)
        .collect::<Vec<i32>>();

    let existing_sku_ids = sqlx::query!(
        "select sku_id from order_items where order_id=$1 and order_goods_id=any($2)",
        order_id,
        &order_goods_ids,
    )
    .fetch_all(db)
    .await
    .map_err(ERPError::DBError)?
    .into_iter()
    .map(|r| r.sku_id)
    .collect::<Vec<i32>>();

    let mut order_items_to_add = vec![];
    for order_goods in order_goods_excel.iter() {
        let this_goods_id = existing_goods_no_to_id
            .get(&order_goods.goods.goods_no)
            .unwrap_or(&0);
        let this_order_goods_id = goods_id_to_order_goods_id.get(this_goods_id).unwrap_or(&0);

        for order_item in order_goods.items.iter() {
            let this_sku_id = goods_id_to_color_to_sku_id
                .get(this_goods_id)
                .unwrap_or(&empty_hashmap)
                .get(&order_item.color)
                .unwrap_or(&0);

            if !existing_sku_ids.contains(this_sku_id) {
                order_items_to_add.push(OrderItemModel {
                    id: 0,
                    order_goods_id: *this_order_goods_id,
                    order_id,
                    sku_id: *this_sku_id,
                    count: order_item.count,
                    unit: Some(order_item.unit.as_deref().unwrap_or("").to_string()),
                    unit_price: Some(order_item.unit_price.unwrap_or(0)),
                    total_price: Some(order_item.total_price.unwrap_or(0)),
                    notes: "".to_string(),
                })
            }
        }
    }

    if !order_items_to_add.is_empty() {
        OrderItemModel::save_to_order_item_table(db, &order_items_to_add).await?;
    }

    Ok(())
}
