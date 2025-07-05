use axum::routing::get;
use utoipa::OpenApi;

use crate::{api::{http_server::AppState, routes::health::health_check}, users::domain::UserRepository};

mod health;

pub fn api_routes<UR: UserRepository + Clone + Sync + Send + 'static>() -> axum::Router<AppState<UR>> {
    axum::Router::new().route("/healthz", get(health_check))
}

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    let doc = health::swagger::ApiDoc::openapi();
    doc
}
