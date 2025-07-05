use axum::{Json, extract::State, http::StatusCode, routing::post};
use utoipa::OpenApi;

use crate::{
    api::{
        domain::{errors::ApiError, response_body::ApiResponseBody},
        http_server::AppState,
    },
    users::{
        domain::{User, UserRepository},
        infrastructure::CreateUserRow,
    },
};

#[utoipa::path(
    post,
    path = "/",
    description = "Create a new user",
    tag = "users",
    responses(
        (status = 201, description = "User created correctly", body = ApiResponseBody<User>),
        (status = 409, description = "Failed to create user", body = ApiResponseBody<String>,
            example = json!({
            "data": "Failed to create user"
        })),
    )
)]
// TODO: This is the endpoint definition, create command and command handler for this and move the
// logic there
pub async fn create_user<UR: UserRepository>(
    State(state): State<AppState<UR>>,
    Json(body): Json<CreateUserRow>,
) -> Result<(StatusCode, Json<ApiResponseBody<User>>), ApiError> {
    let created_user = state.user_repository.create_user(&body).await;

    match created_user {
        Ok(user) => Ok((
            StatusCode::CREATED,
            ApiResponseBody::new(user).into(),
        )),
        Err(_) => Err(ApiError::ConflictError("Failed to create user".to_string())),
    }
}

// Users api routes
pub fn api_routes<UR: UserRepository>() -> axum::Router<AppState<UR>> {
    axum::Router::new().route("/", post(create_user::<UR>))
}

#[derive(OpenApi)]
#[openapi(
    paths(create_user),
    components(schemas(User, ApiError)),
    tags(
        (name = "users", description = "User management API")
    )
)]
pub struct ApiDoc;

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
