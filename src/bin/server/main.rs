use lib::api::http_server::HttpServer;

#[tokio::main]
async fn main() {
    println!("Hello, world! I am changing the code");
    let server = HttpServer::new();

    server.run();
}
