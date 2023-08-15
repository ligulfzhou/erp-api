use crate::constants::STORAGE_FILE_PATH;
use crate::excel::order_umya_excel::read_excel_with_umya;
use crate::model::order::{multi_order_items_no_id_models_to_sql, OrderItemNoIdModel};
use crate::response::api_response::APIEmptyResponse;
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Multipart, State};
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use chrono::{Datelike, Timelike, Utc};
use std::fs;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/page/upload/excel", get(page_upload_file))
        .route("/api/upload/excel", post(import_excel))
        .with_state(state)
}

async fn page_upload_file() -> impl IntoResponse {
    Html(
        r#"
<!DOCTYPE html>
<html>
<body>

<form action="/api/upload" method="post" enctype="multipart/form-data">
    Select image to upload:
    <input type="file" name="fileToUpload" id="fileToUpload">
    <input type="submit" value="Upload Image" name="submit">
</form>

</body>
</html>
    "#,
    )
}

async fn import_excel(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> ERPResult<APIEmptyResponse> {
    let mut id = 0;
    let mut itype = 0;
    let mut file_path: String = "".to_string();

    // 1
    // 这里看着一大堆，其实就是获取三个参数
    // 其中一个参数是 二进制文件，需要保存到本地，并且目录是当前的时间
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        if name == "file" {
            let data = field.bytes().await.unwrap();
            let now = Utc::now();
            let dir_path = format!(
                "{}{}{:02}{:02}",
                STORAGE_FILE_PATH,
                now.year(),
                now.month(),
                now.day()
            );
            let file_name = format!(
                "{}{:02}{:02}{:02}{:02}{:02}.xlsx",
                now.year(),
                now.month(),
                now.day(),
                now.hour(),
                now.minute(),
                now.second()
            );
            fs::create_dir_all(&dir_path)
                .map_err(|_| ERPError::SaveFileFailed(format!("create {} failed", dir_path)))?;
            tracing::info!("Length of `{}` is {} bytes", name, data.len());
            let file_path_full = format!("{}/{}", dir_path, file_name);
            fs::write(&file_path_full, data).map_err(|_| {
                ERPError::SaveFileFailed(format!("create {} failed", file_path_full))
            })?;
            file_path = file_path_full;
        } else if name == "id" {
            let data = String::from_utf8(field.bytes().await.unwrap().to_vec()).unwrap();
            tracing::info!("value of `{}` is: {}", name, data);
            id = data
                .parse::<i32>()
                .map_err(|_| ERPError::ConvertFailed("id".to_string()))?;
        } else if name == "type" {
            let data = String::from_utf8(field.bytes().await.unwrap().to_vec()).unwrap();
            tracing::info!("value of `{}` is: {}", name, data);
            itype = data
                .parse::<i32>()
                .map_err(|_| ERPError::ConvertFailed("type".to_string()))?;
        }
    }

    // 检查文件是否保存成功了
    if file_path.is_empty() {
        return Err(ERPError::Failed("save excel file failed".to_string()));
    }

    // 从excel文件里读 订单信息
    let items = read_excel_with_umya(&file_path);
    tracing::info!("we extract order_items from excel");
    for (index, item) in items.iter().enumerate() {
        println!("{index}: {:?}\n", item);
    }

    let items_to_insert = items
        .iter()
        .map(|item| item.to_order_item_no_id_model(id, 1))
        .collect::<Vec<OrderItemNoIdModel>>();

    let sql = multi_order_items_no_id_models_to_sql(items_to_insert);
    tracing::info!("import sql: {}", sql);
    sqlx::query(&sql).execute(&state.db).await?;

    // for item in items_to_insert {
    //     sqlx::query(
    //         r#"insert into order_items (order_id, sku_id, package_card, package_card_des, count, unit, unit_price, total_price, notes) \
    //         values ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#
    //     )
    //         .bind(item.order_id)
    //         .bind(item.sku_id)
    //         .bind(item.package_card)
    //         .execute(&state.db)
    //         .await?;
    // }

    // // 订单商品列表中获取 goods_no，去db读出这些goods_no的skus
    // let mut goods_nos = items
    //     .iter()
    //     .map(|item| item.goods_no.clone())
    //     .collect::<Vec<String>>();
    // goods_nos.dedup();
    //
    // let goods_no_for_sql = goods_nos
    //     .iter()
    //     .map(|no| format!("'{}'", no.clone()))
    //     .collect::<Vec<String>>()
    //     .join(",");
    // let existing_goods = sqlx::query_as::<_, (i32, String, String)>(&format!(
    //     "select id, goods_no, color from skus where goods_no in ({})",
    //     goods_no_for_sql
    // ))
    // .fetch_all(&state.db)
    // .await
    // .map_err(ERPError::DBError)?;
    //
    // // 对读出的sku放入hashmap，如果有缺失的，则存入 数据库
    // let mut no_to_colors: HashMap<&str, Vec<&str>> = HashMap::new();
    // for (_, goods_no, color) in existing_goods.iter() {
    //     let colors = no_to_colors.entry(goods_no).or_default();
    //     if !colors.contains(&color.as_str()) {
    //         colors.push(color.as_str());
    //     }
    // }
    // tracing::info!("no_to_colors: {:?}", no_to_colors);
    //
    // //
    // println!("existing goods: {:?}", existing_goods);

    Ok(APIEmptyResponse::new())
}
