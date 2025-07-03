use axum::routing::get;
use utoipa::OpenApi;

use crate::api::{http_server::AppState, routes::health::health_check};

mod health;

pub fn api_routes() -> axum::Router<AppState> {
    axum::Router::new().route("/health", get(health_check))
}

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    let doc = health::swagger::ApiDoc::openapi();
    doc
}
