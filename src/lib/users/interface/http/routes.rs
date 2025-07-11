use axum::{Json, extract::State, http::StatusCode, routing::post};
use utoipa::OpenApi;

use crate::{
    api::{
        domain::{
            errors::{ApiError, ApiErrorBody},
            response_body::{ApiResponseBody, TokenResponseBody},
        },
        http_server::AppState,
    },
    protected, require_roles,
    shared::interface::{http::{mw_require_auth, mw_require_role, ValidatedJson}, openapi::security::SecurityAddon},
    users::{
        application::{
            commands::create_user::{
                create_user_command_handler, CreateUserCommand, CreateUserResult
            },
            login::{login_command_handler, LoginCommand},
        },
        domain::{
            user::UserLoginError, LoginTokenService, Role, UserRepository, UserRepositoryError
        },
    },
};

#[utoipa::path(
    post,
    path = "",
    description = "Create a new user",
    tag = "users",
    request_body = CreateUserCommand,
    responses(
        (status = 201, description = "User created correctly", body = ApiResponseBody<CreateUserResult>),
        (status = 409, description = "Failed to create user, user already exists", body = ApiErrorBody,
            example = json!({
            "message": "Failed to create user, user already exists"
        })),
    ),
    security(("bearer_auth" = [])),
)]
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
    tag = "auth",
    request_body = LoginCommand,
    responses(
        (status = 200, description = "User logged in successfully", body = TokenResponseBody),
        (status = 400, description = "Invalid credentials", body = ApiErrorBody,
            example = json!({
            "message": "Invalid username or password"
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
            UserLoginError::InvalidCredentials => Err(ApiError::BadRequestError(err.to_string())),
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
        .route_layer(require_roles!(&[Role::Admin]))
        .route_layer(protected!(state.clone()))
}

#[derive(OpenApi)]
#[openapi(
    paths(create_user),
    modifiers(&SecurityAddon),
    tags(
        (name = "users", description = "User management API")
    )
)]
pub struct ApiDoc;

#[derive(OpenApi)]
#[openapi(
    paths(login_user),
    tags(
        (name = "auth", description = "Authentication API")
    )
)]
pub struct LoginApiDoc;

pub fn combine_openapi() -> utoipa::openapi::OpenApi {
    ApiDoc::openapi()
}
