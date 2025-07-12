use crate::{
    users::{MockLoginTokenService, MockUserRepository},
    utils::functions::hash_password,
};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use lib::{
    api::{http_server::AppState, routes::api_routes},
    users::domain::{Role, User},
};
use std::sync::Arc;
use tower::util::ServiceExt;
use uuid::Uuid;

fn test_app(
    state: AppState<MockUserRepository, MockLoginTokenService>,
) -> Router<AppState<MockUserRepository, MockLoginTokenService>> {
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
}

#[tokio::test]
async fn test_get_user_by_id_success() {
    let user_id = uuid::Uuid::new_v4();
    let user = lib::users::domain::User {
        id: user_id,
        username: "alice".to_string(),
        password: "hashed_password".to_string(),
        role: lib::users::domain::Role::User,
        created_at: Some(chrono::NaiveDateTime::from_timestamp_opt(123456789, 0).unwrap()),
        updated_at: Some(chrono::NaiveDateTime::from_timestamp_opt(987654321, 0).unwrap()),
    };
    let state = AppState {
        user_repository: Arc::new(MockUserRepository {
            user: Some(user.clone()),
            ..MockUserRepository::default()
        }),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/users/{}", user_id))
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["id"], user_id.to_string());
    assert_eq!(json["data"]["username"], "alice");
}

#[tokio::test]
async fn test_get_user_by_id_not_found() {
    let user_id = uuid::Uuid::new_v4();
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/users/{}", user_id))
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_user_by_id_unauthorized() {
    let user_id = uuid::Uuid::new_v4();
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/users/{}", user_id))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_user_by_id_invalid_token() {
    let user_id = uuid::Uuid::new_v4();
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService {
            validation_fail: true,
            ..MockLoginTokenService::default()
        }),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri(&format!("/users/{}", user_id))
        .header("Authorization", "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_user_by_id_malformed_uuid() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/users/not-a-valid-uuid")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_all_users_success() {
    let users = vec![
        lib::users::domain::User {
            id: uuid::Uuid::new_v4(),
            username: "alice".to_string(),
            password: "hashed1".to_string(),
            role: lib::users::domain::Role::User,
            created_at: Some(chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
            updated_at: Some(chrono::NaiveDateTime::from_timestamp_opt(0, 0).unwrap()),
        },
        lib::users::domain::User {
            id: uuid::Uuid::new_v4(),
            username: "bob".to_string(),
            password: "hashed2".to_string(),
            role: lib::users::domain::Role::Admin,
            created_at: Some(chrono::NaiveDateTime::from_timestamp_opt(1, 0).unwrap()),
            updated_at: Some(chrono::NaiveDateTime::from_timestamp_opt(1, 0).unwrap()),
        },
    ];
    let state = AppState {
        user_repository: Arc::new(MockUserRepository {
            users_list: users.clone(),
            ..MockUserRepository::default()
        }),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/users")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
    assert_eq!(json["data"][0]["username"], "alice");
    assert_eq!(json["data"][1]["username"], "bob");
}

#[tokio::test]
async fn test_get_all_users_empty_list() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/users")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_all_users_unauthorized() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/users")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_get_all_users_invalid_token() {
    let state = AppState {
        user_repository: Arc::new(MockUserRepository::default()),
        login_token_service: Arc::new(MockLoginTokenService {
            validation_fail: true,
            ..MockLoginTokenService::default()
        }),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/users")
        .header("Authorization", "Bearer invalid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
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
        .body(Body::from(
            r#"{"username": "newuser", "password": "password123"}"#,
        ))
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
            users_list: vec![],
        }),
        login_token_service: Arc::new(MockLoginTokenService::default()),
    };
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("POST")
        .uri("/users")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer valid_token")
        .body(Body::from(
            r#"{"username": "exists", "password": "password123"}"#,
        ))
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
            users_list: vec![],
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
    let user = User {
        id: Uuid::new_v4(),
        username: "newuser".to_string(),
        password: hash_password("password123"),
        role: Role::User,
        created_at: Some(chrono::naive::NaiveDateTime::default()),
        updated_at: Some(chrono::naive::NaiveDateTime::default()),
    };
    let user_repository = MockUserRepository {
        user_exists: true,
        fail_create: false,
        fail_get: false,
        user: Some(user.clone()),
        users_list: vec![user],
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
        .body(Body::from(
            r#"{"username": "newuser", "password": "password123"}"#,
        ))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
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
        .body(Body::from(
            r#"{"username": "wronguser", "password": "wrongpass"}"#,
        ))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
