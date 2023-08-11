use crate::model::customer::CustomerModel;
use crate::model::order::OrderModel;

#[derive(Debug, Deserialize, Serialize)]
pub struct OrderDto {
    pub id: i32,
    pub customer_id: i32,
    pub customer: String,
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
            customer: customer.name,
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
            customer: "".to_string(),
            customer_address: "".to_string(),
            customer_phone: "".to_string(),
            customer_no: "".to_string(),
            order_no: order.order_no,
            order_date: order.order_date,
            delivery_date: order.delivery_date.unwrap_or(0),
        }
    }
}
