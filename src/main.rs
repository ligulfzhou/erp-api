use std::{
    net::SocketAddr,
    sync::Arc,
};
use axum::{
    Router,
    response::Response,
};
use sqlx::{
    Pool, Postgres,
    postgres::PgPoolOptions,
};
use tower_http::cors::{Any, CorsLayer};

mod error;
mod web;
mod config;
mod model;

pub use self::error::{Result, Error};


#[derive(Debug, Clone)]
pub struct AppState {
    db: Pool<Postgres>,
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
    let pool = match PgPoolOptions::new().max_connections(10).connect(&database_url).await {
        Ok(pool) => pool,
        Err(_err) => std::process::exit(-1),
    };

    let app_state = Arc::new(AppState { db: pool.clone() });
    let cors = CorsLayer::new().allow_origin(Any);

    let routes_all = Router::new()
        .merge(web::routes_login::routes(app_state.clone()))
        .merge(web::routes_hello::routes())
        .merge(web::routes_item::routes(app_state.clone()))
        .fallback_service(web::routes_static::routes())
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8100));
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
