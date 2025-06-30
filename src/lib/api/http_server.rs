pub struct HttpServer {
}

impl HttpServer {
    pub fn new() -> Self {
        HttpServer {}
    }

    pub fn run(&self) {
        println!("HTTP server started");
    }
}
