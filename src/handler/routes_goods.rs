use crate::handler::ListParamToSQLTrait;
use crate::model::goods::{GoodsModel, SKUModel};
use crate::response::api_response::{APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/goods", get(get_goods))
        .route("/api/skus", get(get_skus).post(create_sku))
        .route("/api/sku/update", post(update_sku))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct ListGoodsParam {
    name: Option<String>,
    goods_no: Option<String>,
    plating: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

// impl ListParamTrait for ListGoodsParam {
impl ListParamToSQLTrait for ListGoodsParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = "select * from goods ".to_string();
        let mut where_clauses = vec![];
        if let Some(name) = &self.name {
            where_clauses.push(format!(" name='{}' ", name));
        }
        if let Some(goods_no) = &self.goods_no {
            where_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if let Some(plating) = &self.plating {
            where_clauses.push(format!(" plating='{}' ", plating));
        }
        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(50);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(" offset {} limit {};", offset, page_size));

        sql
    }

    fn to_count_sql(&self) -> String {
        let mut sql = "select count(1) from goods ".to_string();
        let mut where_clauses = vec![];
        if let Some(name) = &self.name {
            where_clauses.push(format!(" name='{}' ", name));
        }
        if let Some(goods_no) = &self.goods_no {
            where_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if let Some(plating) = &self.plating {
            where_clauses.push(format!(" plating='{}' ", plating));
        }
        if where_clauses.len() > 0 {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
            sql.push_str(";");
        }

        sql
    }
}

async fn get_goods(
    State(state): State<Arc<AppState>>,
    Query(list_goods_param): Query<ListGoodsParam>,
) -> ERPResult<APIListResponse<GoodsModel>> {
    let pagination_sql = list_goods_param.to_pagination_sql();
    let goods = sqlx::query_as::<_, GoodsModel>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    let count_sql = list_goods_param.to_count_sql();
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIListResponse::new(goods, total.0 as i32))
}

#[derive(Debug, Deserialize)]
struct ListSKUsParam {
    name: Option<String>,
    goods_no: Option<String>,
    plating: Option<String>,
    color: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl ListParamToSQLTrait for ListSKUsParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = "select * from skus ".to_string();
        let mut where_clauses = vec![];
        if let Some(name) = &self.name {
            where_clauses.push(format!(" name='{}' ", name));
        }
        if let Some(goods_no) = &self.goods_no {
            where_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if let Some(plating) = &self.plating {
            where_clauses.push(format!(" plating='{}' ", plating));
        }
        if let Some(color) = &self.color {
            where_clauses.push(format!(" color='{}' ", color));
        }

        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(50);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(
            " order by id desc offset {} limit {};",
            offset, page_size
        ));

        sql
    }

    fn to_count_sql(&self) -> String {
        let mut sql = "select count(1) from skus ".to_string();
        let mut where_clauses = vec![];
        if let Some(name) = &self.name {
            where_clauses.push(format!(" name='{}' ", name));
        }
        if let Some(goods_no) = &self.goods_no {
            where_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if let Some(plating) = &self.plating {
            where_clauses.push(format!(" plating='{}' ", plating));
        }
        if let Some(color) = &self.color {
            where_clauses.push(format!(" color='{}' ", color));
        }

        if where_clauses.len() > 0 {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
            sql.push_str(";");
        }

        sql
    }
}

async fn get_skus(
    State(state): State<Arc<AppState>>,
    Query(list_skus_param): Query<ListSKUsParam>,
) -> ERPResult<APIListResponse<SKUModel>> {
    let pagination_sql = list_skus_param.to_pagination_sql();
    println!("{pagination_sql}");
    let goods = sqlx::query_as::<_, SKUModel>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    let count_sql = list_skus_param.to_count_sql();
    println!("{count_sql}");
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIListResponse::new(goods, total.0 as i32))
}

#[derive(Debug, Deserialize)]
struct CreateSKUParam {
    image: Option<String>,
    goods_no: Option<String>,
    sku_no: String,
    color: Option<String>,
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateSKUsParam {
    skus: Vec<CreateSKUParam>,
}

impl CreateSKUsParam {
    fn to_sql(&self) -> String {
        let values = self
            .skus
            .iter()
            .map(|sku| {
                format!(
                    "('{}', '{}', '{}', '{}', '{}')",
                    sku.image.as_ref().unwrap_or(&"".to_string()),
                    sku.goods_no.as_ref().unwrap_or(&"".to_string()),
                    sku.sku_no,
                    sku.color.as_ref().unwrap_or(&"".to_string()),
                    sku.notes.as_ref().unwrap_or(&"".to_string())
                )
            })
            .collect::<Vec<String>>()
            .join(", ");

        format!(
            "insert into skus (image, goods_no, sku_no, color, notes) values {}",
            values
        )
    }
}

async fn create_sku(
    State(state): State<Arc<AppState>>,
    Json(create_sku_param): Json<CreateSKUsParam>,
) -> ERPResult<APIEmptyResponse> {
    // let to_insert_sql = String::new();
    if create_sku_param.skus.is_empty() {
        return Err(ERPError::ParamNeeded("skus".to_string()));
    }

    let sku_nos: Vec<String> = create_sku_param
        .skus
        .iter()
        .map(|sku| format!("{}", sku.sku_no))
        .collect();
    let sku_nos_sql = sku_nos.join("', '");

    let mut existing = sqlx::query_as!(
        SKUModel,
        "select * from skus where sku_no in ($1)",
        sku_nos_sql
    )
    .fetch_all(&state.db)
    .await
    .map_err(|err| ERPError::DBError(err))?;

    let existing_sku_nos: Vec<String> = existing
        .iter()
        .map(|sku| format!("{:?}", sku.sku_no))
        .collect();
    let to_insert: Vec<&CreateSKUParam> = create_sku_param
        .skus
        .iter()
        .filter(|&sku| existing_sku_nos.contains(&sku.sku_no.to_string()))
        .into_iter()
        .collect();
    if to_insert.is_empty() {
        return Err(ERPError::AlreadyExists("SKU with sku_no".to_string()));
    }

    let values = to_insert
        .iter()
        .map(|sku| {
            format!(
                "('{}', '{}', '{}', '{}', '{}')",
                sku.image.as_ref().unwrap_or(&"".to_string()),
                sku.goods_no.as_ref().unwrap_or(&"".to_string()),
                sku.sku_no,
                sku.color.as_ref().unwrap_or(&"".to_string()),
                sku.notes.as_ref().unwrap_or(&"".to_string())
            )
        })
        .collect::<Vec<String>>()
        .join(", ");

    let sql = format!(
        "insert into skus (image, goods_no, sku_no, color, notes) values {}",
        values
    );

    sqlx::query(&sql)
        .execute(&state.db)
        .await
        .map_err(|err| ERPError::DBError(err))?;

    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct UpdateSKUParams {
    id: i32,
    image: Option<String>,
    goods_no: Option<String>,
    sku_no: Option<String>,
    color: Option<String>,
    notes: Option<String>,
}

impl UpdateSKUParams {
    fn to_sql(&self) -> String {
        let mut set_clauses = vec![];
        if let Some(image) = &self.image {
            set_clauses.push(format!(" image = '{}' ", image))
        }
        if let Some(goods_no) = &self.goods_no {
            set_clauses.push(format!(" goods_no = '{}' ", goods_no))
        }
        if let Some(sku_no) = &self.sku_no {
            set_clauses.push(format!(" sku_no = '{}' ", sku_no))
        }
        if let Some(color) = &self.color {
            set_clauses.push(format!(" color = '{}' ", color))
        }
        if let Some(notes) = &self.notes {
            set_clauses.push(format!(" notes = '{}' ", notes))
        }

        format!(
            "update skus set {} where id = {}",
            set_clauses.join(","),
            self.id
        )
    }
}

async fn update_sku(
    State(state): State<Arc<AppState>>,
    Json(update_sku_param): Json<UpdateSKUParams>,
) -> ERPResult<APIEmptyResponse> {
    sqlx::query(&update_sku_param.to_sql())
        .execute(&state.db)
        .await?;

    Ok(APIEmptyResponse::new())
}

#[cfg(test)]
mod tests {
    use crate::handler::routes_goods::ListGoodsParam;
    use crate::handler::ListParamToSQLTrait;

    #[test]
    fn test() {
        let params = ListGoodsParam {
            name: Some("name".to_string()),
            goods_no: Some("goods_no".to_string()),
            plating: None,
            page: None,
            page_size: None,
        };
        let sql = params.to_pagination_sql();
        let count_sql = params.to_count_sql();
        println!("{}", params.to_pagination_sql());
        println!("{}", count_sql);
    }
}
