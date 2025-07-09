use axum::routing::{get, post};
use utoipa::OpenApi;

use crate::{
    api::{http_server::AppState, routes::health::health_check},
    users::{self, domain::{LoginTokenService, UserRepository}, interface::http::routes::login_user},
};

mod health;

pub fn api_routes<UR: UserRepository, TS: LoginTokenService>(state: AppState<UR, TS>) -> axum::Router<AppState<UR, TS>>
{
    axum::Router::new()
        .route("/healthz", get(health_check))
        .route("/login", post(login_user))
        .nest("/users", users::interface::http::api_routes::<UR, TS>(state.clone()))
}

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    let doc = health::swagger::ApiDoc::openapi()
        .nest("/users", users::interface::http::ApiDoc::openapi());
    doc
}
