use lib::api::http_server::HttpServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Hello, world! I am changing the code");
    let server = HttpServer::new().await?;

    server.run().await
}
