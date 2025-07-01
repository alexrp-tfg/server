use lib::api::http_server::HttpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = HttpServer::new().await?;

    server.run().await
}
