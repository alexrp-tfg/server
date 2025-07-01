use anyhow::Result;

#[tokio::test]
async fn quick_dev() -> Result<()> {
    let hc = httpc_test::new_client("http://localhost:8000/api")?;

    let _ = hc.do_get("/health")
        .await?
        .print()
        .await;

    Ok(())
}
