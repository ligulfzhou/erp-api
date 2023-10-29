use crate::dto::dto_goods::{GoodsDto, SKUModelDto, SKUModelWithoutImageAndPackageDto};
use crate::model::goods::GoodsModel;
use crate::model::order::{GoodsImagesAndPackageModel, OrderGoodsModel};
use crate::{ERPError, ERPResult};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

pub struct GoodsService {}

impl GoodsService {
    pub fn new() -> GoodsService {
        Self {}
    }
}

// goods related
impl GoodsService {
    pub async fn get_goods_dtos(
        db: &Pool<Postgres>,
        goods_ids: &[i32],
    ) -> ERPResult<Vec<GoodsDto>> {
        let goods_without_images_package = sqlx::query_as!(
            GoodsModel,
            "select * from goods where id = any($1)",
            goods_ids
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?;

        let goods_id_to_images_package =
            OrderGoodsModel::get_multiple_goods_images_and_package(db, &goods_ids)
                .await?
                .into_iter()
                .map(|item| (item.goods_id, item))
                .collect::<HashMap<i32, GoodsImagesAndPackageModel>>();

        let default_images_package = GoodsImagesAndPackageModel::default();

        let mut goodses = vec![];
        for goods in goods_without_images_package.into_iter() {
            let images_package = goods_id_to_images_package
                .get(&goods.id)
                .unwrap_or(&default_images_package);
            goodses.push(GoodsDto::from_goods_and_images_package(
                goods,
                images_package.clone(),
            ));
        }

        Ok(goodses)
    }
}

// sku related
impl GoodsService {
    pub async fn get_sku_dtos(db: &Pool<Postgres>, sku_ids: &[i32]) -> ERPResult<Vec<SKUModelDto>> {
        let skus_no_image_package = sqlx::query_as!(
            SKUModelWithoutImageAndPackageDto,
            r#"
            select
                s.id, s.sku_no, g.customer_no, g.name, g.goods_no, g.id as goods_id,
                g.plating, s.color, s.color2, s.notes
            from skus s, goods g
            where s.goods_id = g.id
                and s.id = any($1)
            "#,
            sku_ids
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?;

        let mut goods_ids = skus_no_image_package
            .iter()
            .map(|item| item.goods_id)
            .collect::<Vec<i32>>();
        goods_ids.dedup();

        let goods_id_to_images_package =
            OrderGoodsModel::get_multiple_goods_images_and_package(db, &goods_ids)
                .await?
                .into_iter()
                .map(|item| (item.goods_id, item))
                .collect::<HashMap<i32, GoodsImagesAndPackageModel>>();

        let default_images_package = GoodsImagesAndPackageModel::default();

        let mut skus = vec![];
        for sku in skus_no_image_package.into_iter() {
            let images_package = goods_id_to_images_package
                .get(&sku.goods_id)
                .unwrap_or(&default_images_package);
            skus.push(SKUModelDto::from_sku_and_images_package(
                sku,
                images_package.clone(),
            ));
        }

        Ok(skus)
    }

    pub async fn get_sku_dtos_with_goods_ids(
        db: &Pool<Postgres>,
        goods_ids: &[i32],
    ) -> ERPResult<Vec<SKUModelDto>> {
        let skus_no_image_package = sqlx::query_as!(
            SKUModelWithoutImageAndPackageDto,
            r#"
            select
                s.id, s.sku_no, g.customer_no, g.name, g.goods_no, g.id as goods_id,
                g.plating, s.color, s.color2, s.notes
            from skus s, goods g
            where s.goods_id = g.id
                and s.goods_id = any($1)
            "#,
            goods_ids
        )
        .fetch_all(db)
        .await
        .map_err(ERPError::DBError)?;

        let goods_id_to_images_package =
            OrderGoodsModel::get_multiple_goods_images_and_package(db, goods_ids)
                .await?
                .into_iter()
                .map(|item| (item.goods_id, item))
                .collect::<HashMap<i32, GoodsImagesAndPackageModel>>();

        let default_images_package = GoodsImagesAndPackageModel::default();

        let mut skus = vec![];
        for sku in skus_no_image_package.into_iter() {
            let images_package = goods_id_to_images_package
                .get(&sku.goods_id)
                .unwrap_or(&default_images_package);
            skus.push(SKUModelDto::from_sku_and_images_package(
                sku,
                images_package.clone(),
            ));
        }

        Ok(skus)
    }
}
