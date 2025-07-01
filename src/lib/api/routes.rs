use axum::{response::Html, routing::get};

use crate::api::http_server::AppState;

pub fn api_routes() -> axum::Router<AppState> {
    axum::Router::new()
        .route("/health", get(health_check))
}

async fn health_check() -> Html<&'static str> {
    Html("<h1>API is healthy</h1>")
}
