use axum::routing::{get, post};
use utoipa::OpenApi;

use crate::{
    api::{http_server::AppState, routes::health::health_check},
    users::{self, domain::UserRepository, interface::http::routes::login_user},
};

mod health;

pub fn api_routes<UR: UserRepository>() -> axum::Router<AppState<UR>>
{
    axum::Router::new()
        .route("/healthz", get(health_check))
        .route("/login", post(login_user))
        .nest("/users", users::interface::http::api_routes::<UR>())
}

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    let doc = health::swagger::ApiDoc::openapi()
        .nest("/users", users::interface::http::ApiDoc::openapi());
    doc
}
