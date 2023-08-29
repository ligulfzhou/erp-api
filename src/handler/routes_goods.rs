use crate::constants::DEFAULT_PAGE_SIZE;
use crate::dto::dto_goods::{GoodsDto, SKUModelDto};
use crate::handler::ListParamToSQLTrait;
use crate::model::goods::{GoodsModel, SKUModel};
use crate::response::api_response::{APIEmptyResponse, APIListResponse};
use crate::{AppState, ERPError, ERPResult};
use axum::extract::{Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_extra::extract::WithRejection;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/goods", get(get_goods))
        .route("/api/skus/search", get(search_skus))
        .route("/api/skus", get(get_skus).post(create_sku)) //.post(create_skus))
        .route("/api/sku/update", post(update_sku))
        .with_state(state)
}

#[derive(Debug, Deserialize)]
struct SearchSkusParam {
    key: String,
    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl ListParamToSQLTrait for SearchSkusParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = format!("select * from skus where goods_no like '%{}%'", self.key);
        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(
            " order by id desc offset {} limit {};",
            offset, page_size
        ));

        tracing::info!("sql: {}", sql);

        sql
    }

    fn to_count_sql(&self) -> String {
        format!(
            "select count(1) from skus where goods_no like '%{}%'",
            self.key
        )
    }
}

async fn search_skus(
    State(state): State<Arc<AppState>>,
    Query(param): Query<SearchSkusParam>,
) -> ERPResult<APIListResponse<SKUModel>> {
    let skus = sqlx::query_as::<_, SKUModel>(&param.to_pagination_sql())
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    let total: (i64,) = sqlx::query_as(&param.to_count_sql())
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(skus, total.0 as i32))
}

#[derive(Debug, Deserialize)]
struct ListGoodsParam {
    goods_no: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl ListParamToSQLTrait for ListGoodsParam {
    fn to_pagination_sql(&self) -> String {
        let mut sql = "select * from goods".to_string();
        let mut where_clauses = vec![];
        if self.goods_no.is_some() && !self.goods_no.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("goods_no='{}'", self.goods_no.as_ref().unwrap()));
        }
        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(" offset {} limit {};", offset, page_size));

        sql
    }

    fn to_count_sql(&self) -> String {
        let mut sql = "select count(1) from goods".to_string();
        let mut where_clauses = vec![];
        if self.goods_no.is_some() && !self.goods_no.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("goods_no='{}'", self.goods_no.as_ref().unwrap()));
        }
        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
            sql.push(';');
        }

        sql
    }
}

async fn get_goods(
    State(state): State<Arc<AppState>>,
    Query(param): Query<ListGoodsParam>,
) -> ERPResult<APIListResponse<GoodsDto>> {
    let pagination_sql = param.to_pagination_sql();
    let goods = sqlx::query_as::<_, GoodsModel>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    if goods.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }

    let goods_ids = goods
        .iter()
        .map(|goods| format!("{}", goods.id))
        .collect::<Vec<String>>()
        .join(",");
    let sql = format!("select * from skus where goods_id in ({});", goods_ids);
    tracing::info!("fetch skus with goods_nos: {} with sql: {}", goods_ids, sql);

    let skus = sqlx::query_as::<_, SKUModel>(&sql)
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    let mut id_to_skus = HashMap::new();
    for sku in skus.iter() {
        id_to_skus
            .entry(sku.goods_id)
            .or_insert(vec![])
            .push(sku.clone());
    }
    tracing::info!("id_to_skus: {:#?}", id_to_skus);

    let goods_dtos = goods
        .iter()
        .map(|item| {
            let its_skus = id_to_skus.get(&item.id).unwrap_or(&vec![]).to_owned();
            GoodsDto::from(item.clone(), its_skus)
        })
        .collect::<Vec<GoodsDto>>();

    let count_sql = param.to_count_sql();
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(goods_dtos, total.0 as i32))
}

#[derive(Debug, Deserialize)]
struct ListSKUsParam {
    name: Option<String>,
    // goods_no: Option<String>,
    sku_no: Option<String>,
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
        if self.name.is_some() && !self.name.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("name='{}'", self.name.as_ref().unwrap()));
        }
        // if self.goods_no.is_some() && !self.goods_no.as_ref().unwrap().is_empty() {
        //     where_clauses.push(format!("goods_no='{}'", self.goods_no.as_ref().unwrap()));
        // }
        if self.sku_no.is_some() && !self.sku_no.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("sku_no='{}'", self.sku_no.as_ref().unwrap()));
        }
        if self.plating.is_some() && !self.plating.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("plating='{}'", self.plating.as_ref().unwrap()));
        }
        if self.color.is_some() && !self.color.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("color='{}'", self.color.as_ref().unwrap()));
        }

        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
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
        if self.name.is_some() && !self.name.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("name='{}'", self.name.as_ref().unwrap()));
        }
        // if self.goods_no.is_some() && !self.goods_no.as_ref().unwrap().is_empty() {
        //     where_clauses.push(format!("goods_no='{}'", self.goods_no.as_ref().unwrap()));
        // }
        if self.sku_no.is_some() && !self.sku_no.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("sku_no='{}'", self.sku_no.as_ref().unwrap()));
        }
        if self.plating.is_some() && !self.plating.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("plating='{}'", self.plating.as_ref().unwrap()));
        }
        if self.color.is_some() && !self.color.as_ref().unwrap().is_empty() {
            where_clauses.push(format!("color='{}'", self.color.as_ref().unwrap()));
        }

        if !where_clauses.is_empty() {
            sql.push_str(" where ");
            sql.push_str(&where_clauses.join(" and "));
        }
        sql.push(';');

        sql
    }
}

async fn get_skus(
    State(state): State<Arc<AppState>>,
    Query(param): Query<ListSKUsParam>,
) -> ERPResult<APIListResponse<SKUModelDto>> {
    let pagination_sql = param.to_pagination_sql();
    tracing::info!("{pagination_sql}");
    let skus = sqlx::query_as::<_, SKUModel>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    if skus.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }

    let goods_ids = skus.iter().map(|sku| sku.goods_id).collect::<Vec<i32>>();
    let goods_ids_str = goods_ids
        .iter()
        .map(|goods_id| format!("{}", goods_id))
        .collect::<Vec<String>>()
        .join(",");

    let goods = sqlx::query_as::<_, (i32, String, String)>(&format!(
        "select id, goods_no, image from goods where id in ({})",
        goods_ids_str
    ))
    .fetch_all(&state.db)
    .await
    .map_err(ERPError::DBError)?;
    tracing::info!("{:?}", goods);

    let id_to_goods_no = goods
        .clone()
        .into_iter()
        .map(|i| (i.0, i.1))
        .collect::<HashMap<i32, String>>();
    let id_to_goods_image = goods
        .into_iter()
        .map(|i| (i.0, i.2))
        .collect::<HashMap<i32, String>>();

    let empty_str = "".to_string();
    let sku_dtos = skus
        .iter()
        .map(|sku| {
            let goods_no = id_to_goods_no.get(&sku.goods_id).unwrap_or(&empty_str);
            let goods_image = id_to_goods_image.get(&sku.goods_id).unwrap_or(&empty_str);
            SKUModelDto::from_sku_goods_no_and_image(sku, goods_no, goods_image)
        })
        .collect::<Vec<SKUModelDto>>();

    let count_sql = param.to_count_sql();
    tracing::info!("{count_sql}");
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(sku_dtos, total.0 as i32))
}

#[derive(Debug, Deserialize, Serialize)]
struct CreateSKUParam {
    goods_id: i32,
    plating: Option<String>,
    color: String,
    notes: Option<String>,
}

impl CreateSKUParam {
    fn to_sql(&self) -> String {
        format!(
            r#"insert into skus (goods_id, plating, color, notes) values ({}, {}, {}, {})"#,
            self.goods_id,
            self.plating.as_ref().unwrap_or(&"".to_string()),
            self.color,
            self.notes.as_ref().unwrap_or(&"".to_string())
        )
    }
}

async fn create_sku(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(param), _): WithRejection<Json<CreateSKUParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    // 查重
    let sku_id = sqlx::query_as::<_, (i32, )>(&format!(
        "select id from skus where goods_id={} and color='{}'",
        param.goods_id, param.color
    ))
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if sku_id.is_some() {
        return Err(ERPError::Collision(format!("已经有了ID为{},颜色为:'{}'的产品了", param.goods_id, param.color)))
    }

    // 插入
    let insert_sql = param.to_sql();
    sqlx::query(&insert_sql).execute(&state.db).await.map_err(ERPError::DBError)?;
    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateSKUParam {
    id: i32,
    image: Option<String>,
    goods_no: Option<String>,
    sku_no: Option<String>,
    color: Option<String>,
    notes: Option<String>,
}

impl UpdateSKUParam {
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
    WithRejection(Json(payload), _): WithRejection<Json<UpdateSKUParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    sqlx::query(&payload.to_sql()).execute(&state.db).await?;

    Ok(APIEmptyResponse::new())
}

#[cfg(test)]
mod tests {
    use crate::handler::routes_goods::ListGoodsParam;
    use crate::handler::ListParamToSQLTrait;
    use anyhow::Result;

    #[test]
    fn test() {
        let params = ListGoodsParam {
            goods_no: Some("goods_no".to_string()),
            page: None,
            page_size: None,
        };
        let sql = params.to_pagination_sql();
        let count_sql = params.to_count_sql();
        assert_eq!(
            "select * from goods where goods_no='goods_no' offset 0 limit 50;",
            sql.as_str()
        );
        assert_eq!(
            "select count(1) from goods where goods_no='goods_no';",
            count_sql.as_str()
        );
    }

    #[tokio::test]
    async fn test_sku() -> Result<()> {
        Ok(())
    }
}
