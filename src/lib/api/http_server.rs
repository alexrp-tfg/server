use anyhow::Context;
use tokio::net;

use crate::api::routes::api_routes;

// State that every handlers share (used for services)
#[derive(Debug, Clone)]
pub struct AppState {}

pub struct HttpServer {
    router: axum::Router,
    listener: net::TcpListener,
}

impl HttpServer {
    pub async fn new() -> anyhow::Result<Self> {
        let state = AppState {};

        let router = axum::Router::new()
            .nest("/api", api_routes())
            .with_state(state);

        // TODO: Change to get port from environment
        let listener = net::TcpListener::bind("0.0.0.0:8000")
            .await
            .with_context(|| format!("Failed to start server on port {}", 8000))?;
        Ok(Self { router, listener })
    }

    /// Runs the HTTP server.
    pub async fn run(self) -> anyhow::Result<()> {
        println!("Listening on port {}", self.listener.local_addr().unwrap());
        axum::serve(self.listener, self.router.into_make_service())
            .await
            .context("received error when running server")?;
        Ok(())
    }
}
