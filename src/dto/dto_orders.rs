use crate::dto::dto_goods::GoodsImagesAndPackage;
use crate::dto::dto_progress::OneProgress;
use crate::model::customer::CustomerModel;
use crate::model::order::OrderModel;
use chrono::NaiveDate;
use sqlx::FromRow;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct OrderDto {
    pub id: i32,
    pub customer_no: String,
    pub order_no: String,
    pub order_date: NaiveDate,
    pub delivery_date: Option<NaiveDate>,
    pub is_return_order: bool,
    pub is_urgent: bool,
    pub is_special: bool,
    pub special_customer: String,
    pub build_by: i32,
}

impl OrderDto {
    pub fn from(order: OrderModel, customer: CustomerModel) -> OrderDto {
        Self {
            id: order.id,
            customer_no: customer.customer_no,
            order_no: order.order_no,
            order_date: order.order_date,
            delivery_date: order.delivery_date,
            is_return_order: order.is_return_order,
            is_urgent: order.is_urgent,
            is_special: order.is_special,
            special_customer: order.special_customer,
            build_by: order.build_by,
        }
    }

    pub fn from_only(order: OrderModel) -> OrderDto {
        Self {
            id: order.id,
            customer_no: order.customer_no,
            order_no: order.order_no,
            order_date: order.order_date,
            delivery_date: order.delivery_date,
            is_return_order: order.is_return_order,
            is_urgent: order.is_urgent,
            is_special: order.is_special,
            special_customer: order.special_customer,
            build_by: order.build_by,
        }
    }
}

type StepCount = HashMap<i32, i32>;
type StepCountUF = HashMap<String, i32>;
type StepIndexCount = HashMap<(i32, i32), i32>;

#[derive(Debug, Serialize)]
pub struct StepIndexCountUF {
    pub step: i32,
    pub index: i32,
    pub count: i32,
}

impl StepIndexCountUF {
    pub fn from_step_index_count(step_index_count: StepIndexCount) -> Vec<StepIndexCountUF> {
        let mut ufs = step_index_count
            .iter()
            .map(|kv| StepIndexCountUF {
                step: kv.0 .0,
                index: kv.0 .1,
                count: *kv.1,
            })
            .collect::<Vec<StepIndexCountUF>>();

        ufs.sort_by_key(|kv| (kv.step, kv.index));

        ufs
    }
}

#[derive(Debug, Serialize)]
pub struct OrderWithStepsDto {
    pub id: i32,
    pub customer_no: String,
    pub order_no: String,
    pub order_date: NaiveDate,
    pub delivery_date: Option<NaiveDate>,
    pub is_return_order: bool,
    pub is_urgent: bool,
    pub is_special: bool,
    pub build_by: i32,
    pub special_customer: String,

    pub done_count: i32,
    pub exception_count: i32,
    pub total_count: i32,
    pub steps: Vec<StepIndexCountUF>,
}

impl OrderWithStepsDto {
    pub fn from_order_dto_and_steps(
        order: OrderDto,
        steps: StepIndexCount,
        done_count: i32,
        exception_count: i32,
        total_count: i32,
    ) -> OrderWithStepsDto {
        Self {
            id: order.id,
            customer_no: order.customer_no,
            order_no: order.order_no,
            order_date: order.order_date,
            delivery_date: order.delivery_date,
            is_return_order: order.is_return_order,
            is_urgent: order.is_urgent,
            is_special: order.is_special,
            build_by: order.build_by,
            special_customer: order.special_customer,
            done_count,
            exception_count,
            total_count,
            steps: StepIndexCountUF::from_step_index_count(steps),
        }
    }
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct OrderGoodsItemDto {
    pub id: i32,
    pub order_id: i32,
    pub order_goods_id: i32,
    pub goods_id: i32,
    pub sku_id: i32,
    pub sku_no: Option<String>,
    pub color: String,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes_images: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Serialize, Clone, FromRow)]
pub struct OrderPlainItemWithoutImagesPackageDto {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    pub goods_no: String,
    pub name: String,
    // pub images: Vec<String>,
    // pub image_des: String,
    // pub package_card: String,
    // pub package_card_des: String,
    pub order_goods_id: i32,
    pub sku_id: i32,
    pub sku_no: Option<String>,
    pub color: String,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes_images: Vec<String>,
    pub notes: String,
}

#[derive(Debug, Serialize, Clone, FromRow)]
pub struct OrderPlainItemDto {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    pub goods_no: String,
    pub name: String,
    pub images: Vec<String>,
    pub image_des: String,
    pub package_card: String,
    pub package_card_des: String,
    pub order_goods_id: i32,
    pub sku_id: i32,
    pub sku_no: Option<String>,
    pub color: String,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes_images: Vec<String>,
    pub notes: String,
}

impl OrderPlainItemDto {
    pub fn from_sku_and_images_package(
        sku: OrderPlainItemWithoutImagesPackageDto,
        images_package: GoodsImagesAndPackage,
    ) -> OrderPlainItemDto {
        Self {
            id: sku.id,
            order_id: sku.order_id,
            goods_id: sku.goods_id,
            goods_no: sku.goods_no,
            name: sku.name,
            images: images_package.images,
            image_des: images_package.image_des,
            package_card: images_package.package_card,
            package_card_des: images_package.package_card_des,
            order_goods_id: sku.order_goods_id,
            sku_id: sku.sku_id,
            sku_no: sku.sku_no,
            color: sku.color,
            count: sku.count,
            unit: sku.unit,
            unit_price: sku.unit_price,
            total_price: sku.total_price,
            notes_images: sku.notes_images,
            notes: sku.notes,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderPlainItemWithCurrentStepDto {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    pub goods_no: String,
    pub name: String,
    pub images: Vec<String>,
    pub package_card: String,
    pub package_card_des: String,
    pub order_goods_id: i32,
    pub sku_id: i32,
    pub sku_no: Option<String>,
    pub color: String,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes: String,

    pub is_next_action: bool,
    pub current_step: i32,
    pub current_index: i32,
    pub current_notes: String,
}

impl OrderPlainItemWithCurrentStepDto {
    pub fn from(
        item: OrderPlainItemDto,
        is_next_action: bool,
        current_step: i32,
        current_index: i32,
        current_notes: &str,
    ) -> OrderPlainItemWithCurrentStepDto {
        Self {
            id: item.id,
            order_id: item.order_id,
            goods_id: item.goods_id,
            goods_no: item.goods_no,
            name: item.name,
            images: item.images,
            package_card: item.package_card,
            package_card_des: item.package_card_des,
            order_goods_id: item.order_goods_id,
            sku_id: item.sku_id,
            sku_no: item.sku_no,
            color: item.color,
            count: item.count,
            unit: item.unit,
            unit_price: item.unit_price,
            total_price: item.total_price,
            notes: item.notes,
            is_next_action,
            current_step,
            current_index,
            current_notes: current_notes.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderGoodsItemWithStepsDto {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    pub order_goods_id: i32,
    pub sku_id: i32,
    pub sku_no: Option<String>,
    pub color: String,
    pub count: i32,
    pub unit: Option<String>,
    pub unit_price: Option<i32>,
    pub total_price: Option<i32>,
    pub notes_images: Vec<String>,
    pub notes: String,

    pub is_next_action: bool,
    pub current_step: i32,
    pub steps: Vec<OneProgress>,
}

impl OrderGoodsItemWithStepsDto {
    pub fn from(
        ogid: OrderGoodsItemDto,
        steps: Vec<OneProgress>,
        is_next_action: bool,
        current_step: i32,
    ) -> OrderGoodsItemWithStepsDto {
        Self {
            id: ogid.id,
            order_id: ogid.order_id,
            goods_id: ogid.goods_id,
            order_goods_id: ogid.order_goods_id,
            sku_id: ogid.sku_id,
            sku_no: ogid.sku_no,
            color: ogid.color,
            count: ogid.count,
            unit: ogid.unit,
            unit_price: ogid.unit_price,
            total_price: ogid.total_price,
            notes_images: ogid.notes_images,
            notes: ogid.notes,
            is_next_action,
            current_step,
            steps,
        }
    }
}

#[derive(Debug, Serialize, FromRow, Clone)]
pub struct OrderGoodsDto {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    pub goods_no: String,
    pub name: String,
    pub images: Vec<String>,
    pub image_des: String,
    // pub plating: String,
    pub package_card: String,
    pub package_card_des: String,
}

#[derive(Debug, Serialize)]
pub struct OrderGoodsWithStepsWithItemStepDto {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    pub goods_no: String,
    pub name: String,
    pub images: Vec<String>,
    pub image_des: String,
    // pub plating: String,
    pub package_card: String,
    pub package_card_des: String,

    pub is_next_action: bool,
    pub current_step: i32, // 如果 is_next_action=false，这里的值则没有意义
    pub current_index: i32,
    pub steps: Vec<StepIndexCountUF>,
    pub items: Vec<OrderGoodsItemWithStepsDto>,
}

impl OrderGoodsWithStepsWithItemStepDto {
    pub fn from_order_with_goods_and_steps_and_items(
        order_goods: OrderGoodsDto,
        steps: StepIndexCount,
        items: Vec<OrderGoodsItemWithStepsDto>,
        is_next_action: bool,
        current_step: i32,
        current_index: i32,
    ) -> OrderGoodsWithStepsWithItemStepDto {
        Self {
            id: order_goods.id,
            order_id: order_goods.order_id,
            goods_id: order_goods.goods_id,
            goods_no: order_goods.goods_no,
            name: order_goods.name,
            images: order_goods.images,
            image_des: order_goods.image_des,
            // plating: order_goods.plating,
            package_card: order_goods.package_card,
            package_card_des: order_goods.package_card_des,
            is_next_action,
            current_step,
            current_index,
            steps: StepIndexCountUF::from_step_index_count(steps),
            items,
        }
    }
}

#[derive(Debug, Serialize)]
struct OrderItemDto {
    id: i32,
    order_id: i32, // -- 订单ID
    sku_id: i32,   // integer not null, -- 商品ID
    // order_goods_id: i32,   // integer not null,
    package_card: String,     // text,    -- 包装卡片    （存在大问题）
    package_card_des: String, //  -- 包装卡片说明 （存在大问题）
    count: i32,               //   integer not null,  - - 数量
    unit: String,             //  text,- - 单位
    unit_price: Option<i32>,  //  integer, - - 单价
    total_price: Option<i32>, //   integer,  - - 总价 / 金额
    notes: String,            //    text - - 备注,
}
