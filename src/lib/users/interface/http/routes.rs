use axum::{extract::State, http::StatusCode, routing::post, Json};
use utoipa::OpenApi;

use crate::{
    api::{
        domain::{errors::ApiError, response_body::ApiResponseBody},
        http_server::AppState,
    },
    users::{
        application::commands::create_user::{create_user_command_handler, CreateUserCommand, CreateUserResult}, domain::{User, UserRepository, UserRepositoryError}, infrastructure::CreateUserRow, interface::http::extractors::validated_json::ValidatedJson
    },
};

#[utoipa::path(
    post,
    path = "/",
    description = "Create a new user",
    tag = "users",
    request_body = CreateUserRow,
    responses(
        (status = 201, description = "User created correctly", body = ApiResponseBody<User>),
        (status = 409, description = "Failed to create user, user already exists", body = ApiResponseBody<String>,
            example = json!({
            "message": "Failed to create user, user already exists"
        })),
    )
)]
pub async fn create_user<UR: UserRepository>(
    State(state): State<AppState<UR>>,
    ValidatedJson(body): ValidatedJson<CreateUserCommand>,
) -> Result<(StatusCode, Json<ApiResponseBody<CreateUserResult>>), ApiError> {
    match create_user_command_handler(body, state.user_repository.as_ref()).await {
        Ok(user) => Ok((
            StatusCode::CREATED,
            ApiResponseBody::new(user).into(),
        )),
        Err(err) => match err {
            UserRepositoryError::UserAlreadyExists => Err(ApiError::ConflictError(err.to_string())),
            UserRepositoryError::InternalServerError => Err(ApiError::InternalServerError(err.to_string())),
        },
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
