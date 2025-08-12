use std::sync::Arc;

use crate::{
    media::{MockMediaRepository, TestTokenService, get_test_user_id},
    utils::test_helpers::*,
};
use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use lib::{api::routes::api_routes, media::domain::MediaFile};
use tower::util::ServiceExt;
use uuid::Uuid;

fn test_app(state: lib::api::http_server::AppState) -> Router<lib::api::http_server::AppState> {
    api_routes(state)
}

#[tokio::test]
async fn test_get_media_files_success() {
    let user_id = get_test_user_id();
    let media_files = vec![
        MediaFile {
            id: Uuid::new_v4(),
            user_id,
            filename: "image1.jpg".to_string(),
            original_filename: "photo1.jpg".to_string(),
            file_size: 1024,
            content_type: "image/jpeg".to_string(),
            file_path: format!("media/{}/image1.jpg", user_id),
            uploaded_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
            thumbnail_path: Some(format!("media/{}/thumb_image1.jpg", user_id)),
        },
        MediaFile {
            id: Uuid::new_v4(),
            user_id,
            filename: "video1.mp4".to_string(),
            original_filename: "movie1.mp4".to_string(),
            file_size: 2048,
            content_type: "video/mp4".to_string(),
            file_path: format!("media/{}/video1.mp4", user_id),
            uploaded_at: Some(chrono::Utc::now().naive_utc()),
            updated_at: Some(chrono::Utc::now().naive_utc()),
            thumbnail_path: Some(format!("media/{}/thumb_video1.jpg", user_id)),
        },
    ];

    let state = create_test_app_state(CreateTestAppStateArguments {
        token_service: Some(Arc::new(TestTokenService)),
        media_repo: Some(MockMediaRepository {
            media_files: media_files.clone(),
            ..MockMediaRepository::default()
        }),
        ..CreateTestAppStateArguments::default()
    });

    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/media")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 2);
    assert_eq!(json["data"][0]["filename"], "image1.jpg");
    assert_eq!(json["data"][1]["filename"], "video1.mp4");

    // Check if thumbnails are present
    assert!(json["data"][0]["thumbnail_path"].is_string());
    assert!(json["data"][1]["thumbnail_path"].is_string());
    assert!(
        json["data"][0]["thumbnail_path"]
            .as_str()
            .unwrap()
            .contains("thumb_image1.jpg")
    );
    assert!(
        json["data"][1]["thumbnail_path"]
            .as_str()
            .unwrap()
            .contains("thumb_video1.jpg")
    );
}

#[tokio::test]
async fn test_get_media_files_empty_list() {
    let state = create_default_test_app_state();
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/media")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["data"].is_array());
    assert_eq!(json["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_get_media_files_unauthorized() {
    let state = create_default_test_app_state();
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("GET")
        .uri("/media")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_media_success() {
    let user_id = get_test_user_id();
    let media_id = Uuid::new_v4();

    let media_file = MediaFile {
        id: media_id,
        user_id,
        filename: "test.jpg".to_string(),
        original_filename: "original.jpg".to_string(),
        file_size: 1024,
        content_type: "image/jpeg".to_string(),
        file_path: format!("media/{}/test.jpg", user_id),
        uploaded_at: Some(chrono::Utc::now().naive_utc()),
        updated_at: Some(chrono::Utc::now().naive_utc()),
        thumbnail_path: None,
    };

    let state = create_test_app_state(CreateTestAppStateArguments {
        token_service: Some(Arc::new(TestTokenService)),
        media_repo: Some(MockMediaRepository {
            saved_media: Some(media_file.clone()),
            ..MockMediaRepository::default()
        }),
        ..CreateTestAppStateArguments::default()
    });

    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("DELETE")
        .uri(&format!("/media/{}", media_id))
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["data"]["success"], true);
    assert_eq!(json["data"]["media_id"], media_id.to_string());
}

#[tokio::test]
async fn test_delete_media_not_found() {
    let media_id = Uuid::new_v4();
    let state = create_default_test_app_state();
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("DELETE")
        .uri(&format!("/media/{}", media_id))
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_media_unauthorized() {
    let media_id = Uuid::new_v4();
    let state = create_default_test_app_state();
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("DELETE")
        .uri(&format!("/media/{}", media_id))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_delete_media_invalid_uuid() {
    let state = create_default_test_app_state();
    let app = test_app(state.clone()).with_state(state);
    let request = Request::builder()
        .method("DELETE")
        .uri("/media/not-a-valid-uuid")
        .header("Authorization", "Bearer valid_token")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_media_invalid_file_type() {
    let state = create_default_test_app_state();
    let app = test_app(state.clone()).with_state(state);

    let boundary = "----formdata-test-boundary";
    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.txt\"\r\nContent-Type: text/plain\r\n\r\ntext content\r\n--{}--\r\n",
        boundary, boundary
    );

    let request = Request::builder()
        .method("POST")
        .uri("/media/upload")
        .header("Authorization", "Bearer valid_token")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_media_no_file() {
    let state = create_default_test_app_state();
    let app = test_app(state.clone()).with_state(state);

    let boundary = "----formdata-test-boundary";
    let body = format!("--{}--\r\n", boundary);

    let request = Request::builder()
        .method("POST")
        .uri("/media/upload")
        .header("Authorization", "Bearer valid_token")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_upload_media_unauthorized() {
    let state = create_default_test_app_state();
    let app = test_app(state.clone()).with_state(state);

    let boundary = "----formdata-test-boundary";
    let body = format!(
        "--{}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"test.jpg\"\r\nContent-Type: image/jpeg\r\n\r\nfake image data\r\n--{}--\r\n",
        boundary, boundary
    );

    let request = Request::builder()
        .method("POST")
        .uri("/media/upload")
        .header(
            "Content-Type",
            format!("multipart/form-data; boundary={}", boundary),
        )
        .body(Body::from(body))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
