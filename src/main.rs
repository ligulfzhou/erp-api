#[macro_use]
extern crate serde;

use axum::{response::Response, Router};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

mod config;
mod constants;
mod error;
mod excel;
mod handler;
mod macros;
mod model;
mod repository;
mod response;
mod service;

pub use self::error::{ERPError, ERPResult};

#[derive(Debug, Clone)]
pub struct AppState {
    db: Pool<Postgres>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let port = std::env::var("PORT")
        .expect("run on which port")
        .parse::<u16>()
        .expect("port should be number");
    println!("{database_url}");

    let pool = match PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
    {
        Ok(pool) => pool,
        Err(_err) => std::process::exit(-1),
    };

    let app_state = Arc::new(AppState { db: pool.clone() });
    let cors = CorsLayer::new().allow_origin(Any);

    let routes_all = Router::new()
        .merge(handler::routes_login::routes(app_state.clone()))
        .merge(handler::routes_order::routes(app_state.clone()))
        .merge(handler::routes_customer::routes(app_state.clone()))
        .merge(handler::routes_goods::routes(app_state.clone()))
        .merge(handler::routes_hello::routes())
        .fallback_service(handler::routes_static::routes())
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    println!("=> Listen on {addr} \n");

    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
}

async fn main_response_mapper(res: Response) -> Response {
    println!("->> {:<12} - main_response_mapper", "res_mapper");

    println!("");
    res
}
