use crate::constants::STEP_TO_DEPARTMENT;
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
        }
    }
}

type StepCount = HashMap<i32, i32>;
type StepCountUF = HashMap<String, i32>;

pub fn to_step_count_user_friendly(sc: StepCount) -> StepCountUF {
    sc.into_iter()
        .map(|item| {
            (
                STEP_TO_DEPARTMENT.get(&item.0).unwrap_or(&"").to_string(),
                item.1,
            )
        })
        .collect::<HashMap<String, i32>>()
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
    pub steps: StepCountUF,
}

impl OrderWithStepsDto {
    pub fn from_order_dto_and_steps(order: OrderDto, steps: StepCount) -> OrderWithStepsDto {
        Self {
            id: order.id,
            customer_no: order.customer_no,
            order_no: order.order_no,
            order_date: order.order_date,
            delivery_date: order.delivery_date,
            is_return_order: order.is_return_order,
            is_urgent: order.is_urgent,
            steps: to_step_count_user_friendly(steps),
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
    pub notes: String,
}

#[derive(Debug, Serialize, Clone, FromRow)]
pub struct OrderPlainItemDto {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    pub goods_no: String,
    pub name: String,
    pub image: String,
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
}

#[derive(Debug, Serialize, Clone)]
pub struct OrderPlainItemWithCurrentStepDto {
    pub id: i32,
    pub order_id: i32,
    pub goods_id: i32,
    pub goods_no: String,
    pub name: String,
    pub image: String,
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
    pub step: String,
}

impl OrderPlainItemWithCurrentStepDto {
    pub fn from(
        item: OrderPlainItemDto,
        is_next_action: bool,
        current_step: i32,
    ) -> OrderPlainItemWithCurrentStepDto {
        Self {
            id: item.id,
            order_id: item.order_id,
            goods_id: item.goods_id,
            goods_no: item.goods_no,
            name: item.name,
            image: item.image,
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
            step: STEP_TO_DEPARTMENT
                .get(&current_step)
                .unwrap_or(&"")
                .to_string(),
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
    pub image: String,
    pub plating: String,
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
    pub image: String,
    pub plating: String,
    pub package_card: String,
    pub package_card_des: String,

    pub is_next_action: bool,
    pub current_step: i32, // 如果 is_next_action=false，这里的值则没有意义
    pub steps: StepCountUF,
    pub items: Vec<OrderGoodsItemWithStepsDto>,
}

impl OrderGoodsWithStepsWithItemStepDto {
    pub fn from_order_with_goods_and_steps_and_items(
        order_goods: OrderGoodsDto,
        steps: StepCount,
        items: Vec<OrderGoodsItemWithStepsDto>,
        is_next_action: bool,
        current_step: i32,
    ) -> OrderGoodsWithStepsWithItemStepDto {
        Self {
            id: order_goods.id,
            order_id: order_goods.order_id,
            goods_id: order_goods.goods_id,
            goods_no: order_goods.goods_no,
            name: order_goods.name,
            image: order_goods.image,
            plating: order_goods.plating,
            package_card: order_goods.package_card,
            package_card_des: order_goods.package_card_des,
            is_next_action,
            current_step,
            steps: to_step_count_user_friendly(steps),
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
