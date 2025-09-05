use axum::routing::{get, post};
use utoipa::{OpenApi, openapi::ServerBuilder};

use crate::{
    api::{http_server::AppState, routes::health::health_check},
    media,
    users::{
        self,
        interface::http::routes::{LoginApiDoc, login_user},
    },
};

pub mod health;

pub fn api_routes(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/healthz", get(health_check))
        .route("/login", post(login_user))
        .nest("/user", users::interface::http::api_routes(state.clone()))
        .nest("/media", media::interface::http::api_routes(state.clone()))
}

pub fn combine_openapi(port: &u16) -> utoipa::openapi::OpenApi {
    let mut doc = health::swagger::ApiDoc::openapi()
        .merge_from(LoginApiDoc::openapi())
        .nest("/users", users::interface::http::ApiDoc::openapi())
        .nest("/media", media::interface::http::ApiDoc::openapi());

    doc.servers = Some(vec![
        ServerBuilder::new()
            .url(format!("http://localhost:{}/api", port))
            .description(Some("Local server"))
            .build(),
    ]);
    doc
}
