use lib::{api::{http_server::AppState, routes::api_routes}, users::domain::{Role, User}};
use uuid::Uuid;
use std::sync::Arc;
use axum::{body::Body, http::{Request, StatusCode}, Router};
use crate::{users::{MockLoginTokenService, MockUserRepository}, utils::functions::hash_password};
use tower::util::ServiceExt;

fn test_app(state: AppState<MockUserRepository, MockLoginTokenService>) -> Router<AppState<MockUserRepository, MockLoginTokenService>> {
    api_routes(state)
}

#[tokio::test]
async fn test_health_check() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/healthz")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "healthy");
}

#[tokio::test]
async fn test_user_registration_success() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer valid_token")
        .body(Body::from(r#"{"username": "newuser", "password": "password123"}"#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
}

#[tokio::test]
async fn test_user_registration_duplicate() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository {
            user_exists: true,
            fail_create: false,
            fail_get: false,
            user: None,
        }),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer valid_token")
        .body(Body::from(r#"{"username": "exists", "password": "password123"}"#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_user_registration_invalid_payload() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository {
            user_exists: true,
            fail_create: false,
            fail_get: false,
            user: None,
        }),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer valid_token")
        .body(Body::from(r#"{"username": ""}"#)) // missing password
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_login_success() {
    let user_repository = MockUserRepository {
        user_exists: true,
        fail_create: false,
        fail_get: false,
        user: Some(User {
            id: Uuid::new_v4(),
            username: "newuser".to_string(),
            password: hash_password("password123"),
            role: Role::User,
            created_at: Some(chrono::naive::NaiveDateTime::default()),
            updated_at: Some(chrono::naive::NaiveDateTime::default()),
        }),
    };
    let state = AppState {
        user_repository: Arc::new(user_repository),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"username": "newuser", "password": "password123"}"#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("token").is_some());
}

#[tokio::test]
async fn test_login_invalid_credentials() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/login")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"username": "wronguser", "password": "wrongpass"}"#))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
