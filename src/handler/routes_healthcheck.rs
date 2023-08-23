use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

pub fn routes() -> Router {
    Router::new().route("/api/healthcheck", get(health_check))
}

async fn health_check() -> impl IntoResponse {
    let json_response = serde_json::json!({
        "code": 0,
        "msg": "success",
    });

    Json(json_response)
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let client = httpc_test::new_client("http://localhost:9100")?;
        client.do_get("/api/healthcheck").await?.print().await?;
        Ok(())
    }
}
