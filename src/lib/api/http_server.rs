use std::env;

use anyhow::Context;
use tokio::net;
use tower_http::trace::TraceLayer;

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
        dotenvy::dotenv().context("Failed to load .env file")?;

        let state = AppState {};

        let router = axum::Router::new()
            .nest("/api", api_routes())
            .with_state(state)
        .layer(TraceLayer::new_for_http());

        let port = env::var("API_PORT").context("Failed to read API_PORT from environment")?
            .parse::<u16>()
            .context("Failed to parse API_PORT as u16")?;

        let listener = net::TcpListener::bind(format!("0.0.0.0:{}", port))
            .await
            .with_context(|| format!("Failed to start server on port {}", port))?;
        Ok(Self { router, listener })
    }

    /// Runs the HTTP server.
    pub async fn run(self) -> anyhow::Result<()> {
        println!("Listening on port {}", self.listener.local_addr().expect("Failed to get local address"));
        axum::serve(self.listener, self.router.into_make_service())
            .await
            .context("received error when running server")?;
        Ok(())
    }
}
