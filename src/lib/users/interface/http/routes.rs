use crate::shared::interface::http::mw_require_role;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
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
    shared::interface::{http::ValidatedJson, openapi::security::SecurityAddon},
    users::{
        application::{
            commands::create_user::{
                CreateUserCommand, CreateUserResult, create_user_command_handler,
            },
            login::{LoginCommand, login_command_handler},
            queries::{
                get_all_users::{GetAllUsersResult, get_all_users_query_handler},
                get_user::{GetUserQuery, GetUserResult, get_user_query_handler},
            },
        },
        domain::{Role, UserRepositoryError, user::UserLoginError},
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
pub async fn create_user(
    State(state): State<AppState>,
    ValidatedJson(body): ValidatedJson<CreateUserCommand>,
) -> Result<(StatusCode, Json<ApiResponseBody<CreateUserResult>>), ApiError> {
    match create_user_command_handler(body, state.user_repository.as_ref()).await {
        Ok(user) => Ok((StatusCode::CREATED, ApiResponseBody::new(user).into())),
        Err(err) => match err {
            UserRepositoryError::UserAlreadyExists => Err(ApiError::ConflictError(err.to_string())),
            UserRepositoryError::InternalServerError => {
                Err(ApiError::InternalServerError(err.to_string()))
            }
            UserRepositoryError::UserNotFound => Err(ApiError::InternalServerError(
                "Internal server error".to_string(),
            )),
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
pub async fn login_user(
    State(state): State<AppState>,
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

#[utoipa::path(
    get,
    path = "",
    description = "Get all users",
    tag = "users",
    responses(
        (status = 200, description = "Users retrieved successfully", body = ApiResponseBody<Vec<GetAllUsersResult>>),
        (status = 500, description = "Internal server error", body = ApiErrorBody,
            example = json!({
            "message": "Internal server error"
        }))
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_all_users(
    State(state): State<AppState>,
) -> Result<(StatusCode, Json<ApiResponseBody<Vec<GetAllUsersResult>>>), ApiError> {
    match get_all_users_query_handler(state.user_repository.as_ref()).await {
        Ok(users) => Ok((StatusCode::OK, ApiResponseBody::new(users).into())),
        Err(_) => Err(ApiError::InternalServerError(
            "Internal server error".to_string(),
        )),
    }
}

#[utoipa::path(
    get,
    path = "/{id}",
    description = "Get user by ID",
    tag = "users",
    params(
        ("id" = uuid::Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User retrieved successfully", body = ApiResponseBody<GetUserResult>),
        (status = 404, description = "User not found", body = ApiErrorBody,
            example = json!({
            "message": "User not found"
        })),
        (status = 500, description = "Internal server error", body = ApiErrorBody,
            example = json!({
            "message": "Internal server error"
        }))
    ),
    security(("bearer_auth" = [])),
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<ApiResponseBody<GetUserResult>>), ApiError> {
    // Validate UUID format
    let id = uuid::Uuid::parse_str(&id)
        .map_err(|_| ApiError::BadRequestError("Invalid user ID format".to_string()))?;

    let query = GetUserQuery { id };
    match get_user_query_handler(query, state.user_repository.as_ref()).await {
        Ok(Some(user)) => Ok((StatusCode::OK, ApiResponseBody::new(user).into())),
        Ok(None) => Err(ApiError::NotFoundError("User not found".to_string())),
        Err(_) => Err(ApiError::InternalServerError(
            "Internal server error".to_string(),
        )),
    }
}

// Users api routes
pub fn api_routes(state: AppState) -> axum::Router<AppState> {
    axum::Router::new()
        .route("/", post(create_user))
        .route_layer(require_roles!(&[Role::Admin]))
        .route("/", get(get_all_users))
        .route("/{id}", get(get_user))
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
