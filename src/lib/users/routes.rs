use axum::{
    extract::State, http::StatusCode, routing::post, Json
};
use utoipa::OpenApi;

use crate::{
    api::http_server::AppState,
    users::{
        domain::{User, UserRepository},
        infrastructure::CreateUserRow,
    },
};

#[utoipa::path(
    post,
    path = "/",
    description = "Create a new user",
    responses(
        (status = 200, description = "User created correctly", body = User),
    )
)]
pub async fn create_user<UR: UserRepository>(
    State(state): State<AppState<UR>>,
    Json(body): Json<CreateUserRow>,
) -> (StatusCode, Json<User>) {
    let user = User {
        id: uuid::Uuid::new_v4(),
        username: body.username,
        password: body.password,
        created_at: None,
        updated_at: None,
    };
    (StatusCode::CREATED, Json(user))
}

// Users api routes
pub fn api_routes<UR: UserRepository>() -> axum::Router<AppState<UR>>
{
    axum::Router::new().route("/", post(create_user::<UR>))
}

#[derive(OpenApi)]
#[openapi(paths(create_user), components(schemas(User)))]
pub struct ApiDoc;

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
