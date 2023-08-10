pub mod routes_customer;
pub mod routes_goods;
pub mod routes_hello;
pub mod routes_login;
pub mod routes_order;
pub mod routes_static;

pub trait ListParamToSQLTrait {
    fn to_pagination_sql(&self) -> String;
    fn to_count_sql(&self) -> String;
}

pub trait CreateOrUpdateParamToSQLTrait {
    fn to_sql(&self) -> String;
}
