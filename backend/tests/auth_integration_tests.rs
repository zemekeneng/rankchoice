use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

mod common;
use common::*;

#[sqlx::test]
async fn test_register_success(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    let user_data = json!({
        "email": "test@example.com",
        "password": "testpassword123",
        "name": "Test User"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(user_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data["success"], true);
    assert!(response_data["data"]["user"]["id"].is_string());
    assert_eq!(response_data["data"]["user"]["email"], "test@example.com");
    assert_eq!(response_data["data"]["user"]["name"], "Test User");
    assert_eq!(response_data["data"]["user"]["role"], "pollster");
    assert!(response_data["data"]["token"].is_string());
    assert!(response_data["data"]["refresh_token"].is_string());
}

#[sqlx::test]
async fn test_register_duplicate_email(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    let user_data = json!({
        "email": "duplicate@example.com",
        "password": "testpassword123",
        "name": "First User"
    });

    // First registration should succeed
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(user_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Second registration with same email should fail
    let duplicate_data = json!({
        "email": "duplicate@example.com",
        "password": "differentpassword",
        "name": "Second User"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(duplicate_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data["success"], false);
    assert_eq!(response_data["error"]["code"], "USER_ALREADY_EXISTS");
}

#[sqlx::test]
async fn test_register_invalid_data(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Test missing email
    let invalid_data = json!({
        "password": "testpassword123",
        "name": "Test User"
    });

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(invalid_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY);

    // Test missing password
    let invalid_data = json!({
        "email": "test@example.com",
        "name": "Test User"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(invalid_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY);
}

#[sqlx::test]
async fn test_login_success(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // First register a user
    let user_data = json!({
        "email": "login@example.com",
        "password": "testpassword123",
        "name": "Login User"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(user_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Now test login
    let login_data = json!({
        "email": "login@example.com",
        "password": "testpassword123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data["success"], true);
    assert_eq!(response_data["data"]["user"]["email"], "login@example.com");
    assert!(response_data["data"]["token"].is_string());
    assert!(response_data["data"]["refresh_token"].is_string());
}

#[sqlx::test]
async fn test_login_invalid_credentials(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Register a user first
    let user_data = json!({
        "email": "creds@example.com",
        "password": "correctpassword",
        "name": "Creds User"
    });

    let _ = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(user_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Test with wrong password
    let login_data = json!({
        "email": "creds@example.com",
        "password": "wrongpassword"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data["success"], false);
    assert_eq!(response_data["error"]["code"], "INVALID_CREDENTIALS");
}

#[sqlx::test]
async fn test_login_nonexistent_user(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "somepassword"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(login_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data["success"], false);
    assert_eq!(response_data["error"]["code"], "INVALID_CREDENTIALS");
}

#[sqlx::test]
async fn test_refresh_token_success(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    // Register and login to get tokens
    let user_data = json!({
        "email": "refresh@example.com",
        "password": "testpassword123",
        "name": "Refresh User"
    });

    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(user_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(register_response.into_body(), usize::MAX).await.unwrap();
    let register_data: Value = serde_json::from_slice(&body).unwrap();
    let refresh_token = register_data["data"]["refresh_token"].as_str().unwrap();

    // Wait to ensure different timestamp
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

    // Test refresh token
    let refresh_data = json!({
        "refresh_token": refresh_token
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(refresh_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data["success"], true);
    assert!(response_data["data"]["token"].is_string());
    // Verify it's a different token
    assert_ne!(response_data["data"]["token"], register_data["data"]["token"]);
}

#[sqlx::test]
async fn test_refresh_token_invalid(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    let refresh_data = json!({
        "refresh_token": "invalid.jwt.token"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(refresh_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    // Should return either 401 Unauthorized or 500 Internal Server Error for invalid token
    assert!(response.status() == StatusCode::UNAUTHORIZED || response.status() == StatusCode::INTERNAL_SERVER_ERROR);

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data["success"], false);
    // The error should contain information about token validation failure
    assert!(response_data["error"]["code"].as_str().is_some());
}

#[sqlx::test]
async fn test_api_response_format(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;

    let user_data = json!({
        "email": "format@example.com",
        "password": "testpassword123",
        "name": "Format User"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/register")
                .header("content-type", "application/json")
                .body(Body::from(user_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();

    // Verify API response format
    assert!(response_data["success"].is_boolean());
    assert!(response_data["data"].is_object() || response_data["data"].is_null());
    assert!(response_data["error"].is_object() || response_data["error"].is_null());
    assert!(response_data["metadata"].is_object());
    assert!(response_data["metadata"]["timestamp"].is_string());
    assert!(response_data["metadata"]["version"].is_string());
} 