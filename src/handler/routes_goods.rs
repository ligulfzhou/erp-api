use crate::constants::DEFAULT_PAGE_SIZE;
use crate::dto::dto_goods::{GoodsDto, SKUModelDto};
use crate::handler::ListParamToSQLTrait;
use crate::model::goods::{GoodsModel, SKUModel};
use crate::response::api_response::{APIDataResponse, APIEmptyResponse, APIListResponse};
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
        .route("/api/sku/detail", get(get_sku_detail)) //.post(create_skus))
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
        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let offset = (page - 1) * page_size;

        let mut sql = format!(
            r#"
            select
                s.id, s.sku_no, s.goods_id, s.color,
                g.name, g.image, g.goods_no, g.plating, s.notes
            from skus s, goods g
            where s.goods_id = g.id
                and g.goods_no like '%{}%' or s.sku_no like '%{}%'
            "#,
            self.key, self.key
        );
        sql.push_str(&format!(
            " order by id desc offset {} limit {};",
            offset, page_size
        ));

        tracing::info!("sql: {}", sql);

        sql
    }

    fn to_count_sql(&self) -> String {
        format!(
            r#"
            select count(1)
            from skus s, goods g
            where s.goods_id = g.id
                and g.goods_no like '%{}%' or s.sku_no like '%{}%'
            "#,
            self.key, self.key
        )
    }
}

async fn search_skus(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<SearchSkusParam>, ERPError>,
) -> ERPResult<APIListResponse<SKUModelDto>> {
    let skus = sqlx::query_as::<_, SKUModelDto>(&param.to_pagination_sql())
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
    WithRejection(Query(param), _): WithRejection<Query<ListGoodsParam>, ERPError>,
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
    goods_no: Option<String>,
    sku_no: Option<String>,
    color: Option<String>,
    customer_no: Option<String>,

    page: Option<i32>,
    #[serde(rename(deserialize = "pageSize"))]
    page_size: Option<i32>,
}

impl ListParamToSQLTrait for ListSKUsParam {
    fn to_pagination_sql(&self) -> String {
        let goods_no = self.goods_no.as_deref().unwrap_or("");
        let sku_no = self.sku_no.as_deref().unwrap_or("");
        let color = self.color.as_deref().unwrap_or("");
        let customer_no = self.customer_no.as_deref().unwrap_or("");
        let mut sql = format!(
            r#"
            select
                s.id, s.sku_no, s.goods_id, s.color, s.color2, s.notes,
                g.name, g.goods_no, g.image, g.plating, g.customer_no
            from skus s, goods g
            where s.goods_id = g.id
            "#,
        );
        if !goods_no.is_empty() {
            sql.push_str(&format!(" and g.goods_no like '%{}%'", goods_no));
        }
        if !sku_no.is_empty() {
            sql.push_str(&format!(" and s.sku_no like '%{}%'", sku_no));
        }
        if !color.is_empty() {
            sql.push_str(&format!(" and s.color like '%{}%'", color));
        }
        if !customer_no.is_empty() {
            sql.push_str(&format!(" and g.customer_no = '{}'", customer_no))
        }

        let page = self.page.unwrap_or(1);
        let page_size = self.page_size.unwrap_or(DEFAULT_PAGE_SIZE);
        let offset = (page - 1) * page_size;
        sql.push_str(&format!(
            " order by s.id desc offset {} limit {};",
            offset, page_size
        ));

        sql
    }

    fn to_count_sql(&self) -> String {
        let goods_no = self.goods_no.as_deref().unwrap_or("");
        let sku_no = self.sku_no.as_deref().unwrap_or("");
        let color = self.color.as_deref().unwrap_or("");
        let customer_no = self.customer_no.as_deref().unwrap_or("");
        let mut sql = format!(
            r#"
            select count(1)
            from skus s, goods g
            where s.goods_id = g.id
            "#,
        );
        if !goods_no.is_empty() {
            sql.push_str(&format!(" and g.goods_no like '%{}%'", goods_no));
        }
        if !sku_no.is_empty() {
            sql.push_str(&format!(" and s.sku_no like '%{}%'", sku_no));
        }
        if !color.is_empty() {
            sql.push_str(&format!(" and s.color like '%{}%'", color));
        }
        if !customer_no.is_empty() {
            sql.push_str(&format!(" and g.customer_no = '{}'", customer_no))
        }

        sql
    }
}

async fn get_skus(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<ListSKUsParam>, ERPError>,
) -> ERPResult<APIListResponse<SKUModelDto>> {
    let pagination_sql = param.to_pagination_sql();
    tracing::info!("{pagination_sql}");
    let skus = sqlx::query_as::<_, SKUModelDto>(&pagination_sql)
        .fetch_all(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    if skus.is_empty() {
        return Ok(APIListResponse::new(vec![], 0));
    }

    let count_sql = param.to_count_sql();
    tracing::info!("{count_sql}");
    let total: (i64,) = sqlx::query_as(&count_sql)
        .fetch_one(&state.db)
        .await
        .map_err(ERPError::DBError)?;

    Ok(APIListResponse::new(skus, total.0 as i32))
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
            r#"insert into skus (goods_id, color, notes) values ({}, '{}', '{}')"#,
            self.goods_id,
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
    let sku_id = sqlx::query_as::<_, (i32,)>(&format!(
        "select id from skus where goods_id={} and color='{}'",
        param.goods_id, param.color
    ))
    .fetch_optional(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    if sku_id.is_some() {
        return Err(ERPError::Collision(format!(
            "已经有了ID为{},颜色为:'{}'的产品了",
            param.goods_id, param.color
        )));
    }

    // 插入
    state.execute_sql(&param.to_sql()).await?;
    Ok(APIEmptyResponse::new())
}

#[derive(Debug, Deserialize)]
struct SkuDetailParam {
    id: i32,
}

async fn get_sku_detail(
    State(state): State<Arc<AppState>>,
    WithRejection(Query(param), _): WithRejection<Query<SkuDetailParam>, ERPError>,
) -> ERPResult<APIDataResponse<SKUModelDto>> {
    let sku_dto = sqlx::query_as!(
        SKUModelDto,
        r#"
        select
            s.id, s.sku_no, g.name, g.goods_no, s.goods_id,
            g.image, g.plating, s.color, s.color2, s.notes, g.customer_no
        from skus s, goods g
        where s.goods_id = g.id and s.id = $1;
        "#,
        param.id
    )
    .fetch_one(&state.db)
    .await
    .map_err(ERPError::DBError)?;

    Ok(APIDataResponse::new(sku_dto))
}

#[derive(Debug, Deserialize, Serialize)]
struct UpdateSKUParam {
    id: i32,
    name: Option<String>,
    image: Option<String>,
    goods_id: i32,
    goods_no: Option<String>,
    sku_no: Option<String>,
    color: Option<String>,
    plating: Option<String>,
    notes: Option<String>,
}

impl UpdateSKUParam {
    fn to_sqls(&self) -> Vec<String> {
        // let update_goods_sql = format!("");
        let mut update_set_clauses = vec![];
        let name = self.name.as_deref().unwrap_or("");
        let image = self.image.as_deref().unwrap_or("");
        let sku_no = self.sku_no.as_deref().unwrap_or("");
        let goods_no = self.goods_no.as_deref().unwrap_or("");
        let color = self.color.as_deref().unwrap_or("");
        let plating = self.plating.as_deref().unwrap_or("");
        let notes = self.notes.as_deref().unwrap_or("");

        if !name.is_empty() {
            update_set_clauses.push(format!(" name='{}' ", name))
        }
        if !image.is_empty() {
            update_set_clauses.push(format!(" image='{}' ", image));
        }
        if !goods_no.is_empty() {
            update_set_clauses.push(format!(" goods_no='{}' ", goods_no));
        }
        if !plating.is_empty() {
            update_set_clauses.push(format!(" plating='{}' ", plating));
        }
        let update_goods_sql = match update_set_clauses.len() {
            0 => "".to_string(),
            _ => format!(
                "update goods set {} where id = {}",
                update_set_clauses.join(","),
                self.goods_id
            ),
        };

        let mut set_clauses = vec![];
        if !sku_no.is_empty() {
            set_clauses.push(format!(" sku_no = '{}' ", sku_no))
        }
        if !color.is_empty() {
            set_clauses.push(format!(" color = '{}' ", color))
        }
        if !notes.is_empty() {
            set_clauses.push(format!(" notes = '{}' ", notes))
        }
        let update_item_sql = match set_clauses.len() {
            0 => "".to_string(),
            _ => format!(
                "update skus set {} where id = {}",
                set_clauses.join(","),
                self.id
            ),
        };

        vec![update_goods_sql, update_item_sql]
    }
}

async fn update_sku(
    State(state): State<Arc<AppState>>,
    WithRejection(Json(payload), _): WithRejection<Json<UpdateSKUParam>, ERPError>,
) -> ERPResult<APIEmptyResponse> {
    let sqls = payload.to_sqls();
    for sql in sqls.into_iter() {
        if !sql.is_empty() {
            state.execute_sql(&sql).await?;
        }
    }
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
