use anyhow::Result;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let client = httpc_test::new_client("http://localhost:8080")?;

    client.do_get("/hello").await?.print().await?;

    client.do_get("/src/main.rs").await?.print().await?;

    Ok(())
}
