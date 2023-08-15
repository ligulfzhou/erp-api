use crate::model::customer::CustomerModel;
use crate::model::order::{OrderItemModel, OrderModel};

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderDto {
    pub id: i32,
    pub customer_id: i32,
    pub customer_name: String,
    pub customer_address: String,
    pub customer_phone: String,
    pub customer_no: String,
    pub order_no: String,
    pub order_date: i32,
    pub delivery_date: i32,
}

impl OrderDto {
    pub fn from(order: OrderModel, customer: CustomerModel) -> OrderDto {
        Self {
            id: order.id,
            customer_id: customer.id,
            customer_name: customer.name,
            customer_address: customer.address.unwrap_or("".to_string()),
            customer_phone: customer.phone.unwrap_or("".to_string()),
            customer_no: customer.customer_no,
            order_no: order.order_no,
            order_date: order.order_date,
            delivery_date: order.delivery_date.unwrap_or(0),
        }
    }

    pub fn from_only(order: OrderModel) -> OrderDto {
        Self {
            id: order.id,
            customer_id: order.customer_id,
            customer_name: "".to_string(),
            customer_address: "".to_string(),
            customer_phone: "".to_string(),
            customer_no: "".to_string(),
            order_no: order.order_no,
            order_date: order.order_date,
            delivery_date: order.delivery_date.unwrap_or(0),
        }
    }
}

#[derive(Debug, Serialize)]
struct OrderItemDto {
    id: i32,
    order_id: i32, // -- 订单ID
    sku_id: i32,   // integer not null, -- 商品ID
    // order_goods_id: i32,      // integer not null,
    package_card: String,     // text,    -- 包装卡片    （存在大问题）
    package_card_des: String, //  -- 包装卡片说明 （存在大问题）
    count: i32,               //   integer not null,  - - 数量
    unit: String,             //  text,- - 单位
    unit_price: Option<i32>,  //  integer, - - 单价
    total_price: Option<i32>, //   integer,  - - 总价 / 金额
    notes: String,            //    text - - 备注,
}

impl OrderItemDto {
    fn from(order_item: OrderItemModel) -> OrderItemDto {
        Self {
            id: order_item.id,
            order_id: order_item.order_id,
            sku_id: order_item.sku_id,
            // todo
            package_card: order_item.package_card.unwrap_or("".to_string()),
            // todo
            package_card_des: order_item.package_card_des.unwrap_or("".to_string()),
            count: order_item.count,
            unit: order_item.unit.unwrap_or("".to_string()),
            unit_price: order_item.unit_price,
            total_price: order_item.total_price,
            notes: order_item.notes.unwrap_or("".to_string()),
        }
    }
}