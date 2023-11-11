use crate::common::string::is_empty_string_vec;
use crate::error::ERPResult;
use crate::model::goods::SKUModel;
use crate::model::order::{
    ExcelOrderGoods, ExcelOrderGoodsWithItems, OrderGoodsModel, OrderInfo, OrderItemExcel,
    OrderItemModel,
};
use crate::ERPError;
use itertools::Itertools;
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

pub fn convert_index_vec_order_item_excel_to_vec_excel_order_goods_with_items(
    index_to_order_item_excel: HashMap<i32, Vec<OrderItemExcel>>,
    no_goods_no: bool, // template_id: i32
) -> ERPResult<Vec<ExcelOrderGoodsWithItems>> {
    let empty_order_item_excel_vec: Vec<OrderItemExcel> = vec![];
    let mut res = vec![];
    for index in index_to_order_item_excel.keys().sorted() {
        let items = index_to_order_item_excel
            .get(index)
            .unwrap_or(&empty_order_item_excel_vec);

        if !no_goods_no {
            // 检查数据是否有问题(goods_no至少有一个值）
            let mut goods_nos = items
                .iter()
                .map(|item| item.goods_no.as_str())
                .collect::<Vec<&str>>();
            tracing::info!("goods_nos: {goods_nos:?}");

            if is_empty_string_vec(&goods_nos) {
                return Err(ERPError::ExcelError(format!(
                    "Excel内序号#{index},没有读到商品编号"
                )));
            }

            goods_nos.dedup();

            if goods_nos.len() > 1 {
                return Err(ERPError::ExcelError(format!(
                    "请检查Excel内序号#{index}，有重复数据, 或者序号重复"
                )));
            }
        } else {
            // todo
            // 没有goods_no
        }

        // todo: 检查数据是否有问题
        // let mut goods_nos = items
        //     .iter()
        //     .map(|item| item.goods_no.as_str())
        //     .collect::<Vec<&str>>();
        //
        // println!("goods_no: {:?}", goods_nos);
        // goods_nos.dedup();
        // println!("goods_no: {:?}", goods_nos);
        // if goods_nos.len() > 1 {
        //     return Err(ERPError::ExcelError(format!(
        //         "Excel内序号#{index}可能重复,或者有多余总计的行"
        //     )));
        // }

        let goods = OrderItemExcel::pick_up_excel_goods(items);
        tracing::info!("pick_up_excel_goods: {:?}", goods);
        let excel_order_goods_with_items = ExcelOrderGoodsWithItems {
            goods,
            items: items.clone(),
        };
        res.push(excel_order_goods_with_items);
    }

    Ok(res)
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
    tracing::info!("existing_id_goods_no: {:?}", existing_goods_no_to_id);

    let to_add_goods_nos = goods_nos
        .iter()
        .filter(|item| !existing_goods_no_to_id.contains_key(*item))
        .map(|item| item.as_str())
        .collect::<Vec<&str>>();

    tracing::info!(
        "to_add_goods_nos: {:?}, len: {}",
        to_add_goods_nos,
        to_add_goods_nos.len()
    );

    // todo: 添加 和 修改（已经存在，则去修改原先数据【主要是修改图片和package】）
    if !to_add_goods_nos.is_empty() {
        tracing::info!("order_goods_excel: {:?}", order_goods_excel);
        let to_add_goods = order_goods_excel
            .iter()
            .filter(|item| to_add_goods_nos.contains(&item.goods.goods_no.as_str()))
            .map(|item| item.goods.clone())
            .collect::<Vec<ExcelOrderGoods>>();

        tracing::info!(
            "to_add_goods: {:?}, len: {}",
            to_add_goods,
            to_add_goods.len()
        );

        let new_goods_no_to_id =
            ExcelOrderGoods::insert_into_goods_table(db, &to_add_goods, &order_info.customer_no)
                .await?;

        tracing::info!("new_goods_no_to_id: {:?}", new_goods_no_to_id);

        new_goods_no_to_id.into_iter().for_each(|(goods_no, id)| {
            existing_goods_no_to_id.insert(goods_no, id);
        });
    }
    tracing::info!("goods_no_to_id: {:?}", existing_goods_no_to_id);

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
    skus.into_iter().for_each(|sku| {
        goods_id_to_color_to_sku_id
            .entry(sku.goods_id)
            .or_insert(HashMap::new())
            .insert(sku.color, sku.id);
    });

    let mut skus_to_add = vec![];
    let empty_hashmap: HashMap<String, i32> = HashMap::new();

    order_goods_excel.iter().for_each(|order_goods| {
        let goods_id = existing_goods_no_to_id
            .get(&order_goods.goods.goods_no)
            .unwrap_or(&0);

        tracing::info!("goods_id: {}", goods_id);
        order_goods.items.iter().for_each(|order_goods_sku| {
            if !goods_id_to_color_to_sku_id
                .get(goods_id)
                .unwrap_or(&empty_hashmap)
                .contains_key(&order_goods_sku.color)
            {
                skus_to_add.push(SKUModel {
                    id: 0,
                    goods_id: *goods_id,
                    sku_no: order_goods_sku.sku_no.as_deref().unwrap_or("").to_string(),
                    color: order_goods_sku.color.to_string(),
                    color2: order_goods_sku.color_2.as_deref().unwrap_or("").to_string(),
                    plating: order_goods_sku.plating.clone(),
                    notes: None,
                })
            }
        });
    });

    tracing::info!("skus_to_add: {:?}", skus_to_add);
    if !skus_to_add.is_empty() {
        let new_skus = ExcelOrderGoods::insert_into_skus_table(db, &skus_to_add).await?;
        new_skus.into_iter().for_each(|new_sku| {
            goods_id_to_color_to_sku_id
                .entry(new_sku.goods_id)
                .or_insert(HashMap::new())
                .insert(new_sku.color, new_sku.id);
        });
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
        order_goods_excel.iter().for_each(|order_goods| {
            let this_goods_id = existing_goods_no_to_id
                .get(&order_goods.goods.goods_no)
                .unwrap_or(&0);
            if !goods_id_to_order_goods_id.contains_key(this_goods_id) {
                to_add_order_goods.push(OrderGoodsModel {
                    id: 0,
                    index: order_goods.goods.index,
                    // todo
                    images: order_goods.goods.images.clone(),
                    image_des: order_goods
                        .goods
                        .image_des
                        .as_deref()
                        .unwrap_or("")
                        .to_string(),
                    package_card: order_goods
                        .goods
                        .package_card
                        .as_deref()
                        .unwrap_or("")
                        .to_string(),
                    package_card_des: order_goods
                        .goods
                        .package_card_des
                        .as_deref()
                        .unwrap_or("")
                        .to_string(),
                    order_id,
                    goods_id: *this_goods_id,
                })
            }
        });
        if !to_add_order_goods.is_empty() {
            let new_order_goods = OrderGoodsModel::add_rows(db, &to_add_order_goods).await?;
            new_order_goods.into_iter().for_each(|new_order_good| {
                goods_id_to_order_goods_id.insert(new_order_good.goods_id, new_order_good.id);
            });
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
    order_goods_excel.iter().for_each(|order_goods| {
        let this_goods_id = existing_goods_no_to_id
            .get(&order_goods.goods.goods_no)
            .unwrap_or(&0);
        let this_order_goods_id = goods_id_to_order_goods_id.get(this_goods_id).unwrap_or(&0);

        order_goods.items.iter().for_each(|order_item| {
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
                    notes_images: order_item.notes_images.clone(),
                    notes: order_item.notes.as_deref().unwrap_or("").to_string(),
                })
            }
        });
    });

    if !order_items_to_add.is_empty() {
        OrderItemModel::save_to_order_item_table(db, &order_items_to_add).await?;
    }

    Ok(())
}
