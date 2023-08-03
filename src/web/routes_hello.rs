use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    Router,
    routing::get
};
use serde::Deserialize;


pub fn routes() -> Router {
    Router::new()
        .route("/api/hello", get(handler_hello))
        .route("/api/hello2/:username", get(handler_hello2))
        .route("/api/hello3", get(handler_hello3))
}

async fn handler_hello() -> impl IntoResponse {
    Html("hello world")
}

async fn handler_hello2(Path(username): Path<String>) -> impl IntoResponse {
    Html(format!("hello {:?}", username))
}

#[derive(Debug, Deserialize)]
struct Hello {
    username: String,
}

async fn handler_hello3(Query(hello): Query<Hello>) -> impl IntoResponse {
    Html(format!("hello {:?}", hello.username))
}
