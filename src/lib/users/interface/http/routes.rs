use axum::{Json, extract::State, http::StatusCode, middleware, routing::post};
use utoipa::OpenApi;

use crate::{
    api::{
        domain::{
            errors::{ApiError, ApiErrorBody},
            response_body::{ApiResponseBody, TokenResponseBody},
        },
        http_server::AppState,
    },
    shared::interface::http::{ValidatedJson, mw_require_auth},
    users::{
        application::{
            commands::create_user::{
                CreateUserCommand, CreateUserResult, create_user_command_handler,
            },
            login::{LoginCommand, login_command_handler},
        },
        domain::{LoginTokenService, UserRepository, UserRepositoryError, user::UserLoginError},
    },
};

#[utoipa::path(
    post,
    path = "/",
    description = "Create a new user",
    tag = "users",
    request_body = CreateUserCommand,
    responses(
        (status = 201, description = "User created correctly", body = ApiResponseBody<CreateUserResult>),
        (status = 409, description = "Failed to create user, user already exists", body = ApiErrorBody,
            example = json!({
            "message": "Failed to create user, user already exists"
        })),
    )
)]
// TODO: Implement auth middleware for protected endpoints
pub async fn create_user<UR: UserRepository, TS: LoginTokenService>(
    State(state): State<AppState<UR, TS>>,
    ValidatedJson(body): ValidatedJson<CreateUserCommand>,
) -> Result<(StatusCode, Json<ApiResponseBody<CreateUserResult>>), ApiError> {
    match create_user_command_handler(body, state.user_repository.as_ref()).await {
        Ok(user) => Ok((StatusCode::CREATED, ApiResponseBody::new(user).into())),
        Err(err) => match err {
            UserRepositoryError::UserAlreadyExists => Err(ApiError::ConflictError(err.to_string())),
            UserRepositoryError::InternalServerError => {
                Err(ApiError::InternalServerError(err.to_string()))
            }
        },
    }
}

// Login endpoint
#[utoipa::path(
    post,
    path = "/login",
    description = "Login a user",
    tag = "users",
    request_body = LoginCommand,
    responses(
        (status = 200, description = "User logged in successfully", body = TokenResponseBody),
        (status = 401, description = "Invalid credentials", body = ApiErrorBody,
            example = json!({
            "message": "Invalid credentials"
        })),
        (status = 500, description = "Internal server error", body = ApiErrorBody,
            example = json!({
            "message": "Internal server error"
        }))
    )
)]
pub async fn login_user<UR: UserRepository, TS: LoginTokenService>(
    State(state): State<AppState<UR, TS>>,
    ValidatedJson(body): ValidatedJson<LoginCommand>,
) -> Result<(StatusCode, Json<TokenResponseBody>), ApiError> {
    match login_command_handler(
        body,
        state.user_repository.as_ref(),
        state.login_token_service.as_ref(),
    )
    .await
    {
        Ok(result) => Ok((StatusCode::OK, Json(TokenResponseBody::new(result)))),
        Err(err) => match err {
            UserLoginError::InvalidCredentials => Err(ApiError::UnauthorizedError(err.to_string())),
            UserLoginError::InternalServerError(msg) => Err(ApiError::InternalServerError(msg)),
            UserLoginError::InvalidToken => Err(ApiError::UnauthorizedError(err.to_string())),
        },
    }
}

// Users api routes
pub fn api_routes<UR: UserRepository, TS: LoginTokenService>(
    state: AppState<UR, TS>,
) -> axum::Router<AppState<UR, TS>> {
    axum::Router::new()
        .route("/", post(create_user::<UR, TS>))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            mw_require_auth::<UR, TS>,
        ))
}

#[derive(OpenApi)]
#[openapi(
    paths(create_user, login_user),
    components(schemas(CreateUserCommand, ApiError)),
    tags(
        (name = "users", description = "User management API")
    )
)]
pub struct ApiDoc;

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
