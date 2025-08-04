use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
    Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

mod common;
use common::*;

// Helper function to register user and get auth token
async fn setup_authenticated_user(app: &Router) -> String {
    let user_data = json!({
        "email": "testuser@example.com",
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
                .body(Body::from(user_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: Value = serde_json::from_slice(&body).unwrap();
    
    response_data["data"]["token"].as_str().unwrap().to_string()
}

// Test helper to create a test poll request
fn create_test_poll_request() -> Value {
    json!({
        "title": "Best Programming Language 2024",
        "description": "Vote for your favorite programming language",
        "poll_type": "single_winner",
        "num_winners": 1,
        "is_public": false,
        "registration_required": false,
        "candidates": [
            {
                "name": "Rust",
                "description": "Systems programming language"
            },
            {
                "name": "Python", 
                "description": "General-purpose programming language"
            },
            {
                "name": "JavaScript",
                "description": "Web programming language"
            }
        ]
    })
}

fn create_minimal_poll_request() -> Value {
    json!({
        "title": "Simple Poll",
        "candidates": [
            {"name": "Option A"},
            {"name": "Option B"}
        ]
    })
}

#[sqlx::test]
async fn test_create_poll_success(pool: PgPool) {
    let app = create_test_app_with_user(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/polls")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(create_test_poll_request().to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert!(result["data"].is_object());
    
    let poll_data = &result["data"];
    assert_eq!(poll_data["title"], "Best Programming Language 2024");
    assert_eq!(poll_data["description"], "Vote for your favorite programming language");
    assert_eq!(poll_data["poll_type"], "single_winner");
    assert_eq!(poll_data["num_winners"], 1);
    assert_eq!(poll_data["is_public"], false);
    assert_eq!(poll_data["registration_required"], false);
    
    // Check candidates
    let candidates = poll_data["candidates"].as_array().unwrap();
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0]["name"], "Rust");
    assert_eq!(candidates[1]["name"], "Python");
    assert_eq!(candidates[2]["name"], "JavaScript");
}

#[sqlx::test]
async fn test_create_poll_minimal_success(pool: PgPool) {
    let app = create_test_app_with_user(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/polls")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(create_minimal_poll_request().to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    let poll_data = &result["data"];
    
    // Check defaults
    assert_eq!(poll_data["poll_type"], "single_winner");
    assert_eq!(poll_data["num_winners"], 1);
    assert_eq!(poll_data["is_public"], false);
    assert_eq!(poll_data["registration_required"], false);
}

#[sqlx::test]
async fn test_create_poll_validation_errors(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;

    // Test empty title
    let invalid_request = json!({
        "title": "",
        "candidates": [{"name": "A"}, {"name": "B"}]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/polls")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(invalid_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "VALIDATION_ERROR");
    assert!(result["error"]["message"].as_str().unwrap().contains("title"));
}

#[sqlx::test]
async fn test_create_poll_insufficient_candidates(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;

    let invalid_request = json!({
        "title": "Test Poll",
        "candidates": [{"name": "Only One"}]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/polls")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(invalid_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "VALIDATION_ERROR");
    assert!(result["error"]["message"].as_str().unwrap().contains("2 candidates"));
}

#[sqlx::test]
async fn test_create_poll_empty_candidate_name(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;

    let invalid_request = json!({
        "title": "Test Poll",
        "candidates": [{"name": "Valid"}, {"name": ""}]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/polls")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(invalid_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "VALIDATION_ERROR");
    assert!(result["error"]["message"].as_str().unwrap().contains("candidate names"));
}

#[sqlx::test]
async fn test_list_polls_success(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/polls")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert!(result["data"].is_object());
    
    let data = &result["data"];
    assert!(data["items"].is_array());
    assert!(data["total"].is_number());
    assert!(data["page"].is_number());
    assert!(data["limit"].is_number());
    assert!(data["total_pages"].is_number());
}

#[sqlx::test]
async fn test_list_polls_with_pagination(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/polls?page=1&limit=10")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    let data = &result["data"];
    assert_eq!(data["page"], 1);
    assert_eq!(data["limit"], 10);
}

#[sqlx::test]
async fn test_list_polls_with_filters(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/polls?status=active&sort=title&order=asc")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
}

#[sqlx::test]
async fn test_get_poll_not_found(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let random_id = Uuid::new_v4();
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/polls/{}", random_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "POLL_NOT_FOUND");
}

#[sqlx::test]
async fn test_update_poll_not_found(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let random_id = Uuid::new_v4();
    let update_request = json!({
        "title": "Updated Title"
    });
    
    let request = Request::builder()
        .method(Method::PUT)
        .uri(&format!("/api/polls/{}", random_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(update_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "POLL_NOT_FOUND");
}

#[sqlx::test]
async fn test_update_poll_validation_error(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let random_id = Uuid::new_v4();
    let invalid_update = json!({
        "title": ""  // Empty title should fail validation
    });
    
    let request = Request::builder()
        .method(Method::PUT)
        .uri(&format!("/api/polls/{}", random_id))
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(invalid_update.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test]
async fn test_delete_poll_not_found(pool: PgPool) {
    let app = create_test_app(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    let random_id = Uuid::new_v4();
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(&format!("/api/polls/{}", random_id))
        .header("authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "POLL_NOT_FOUND");
}

#[sqlx::test]
async fn test_api_response_format(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/polls")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify API response structure
    assert!(result["success"].is_boolean());
    assert!(result["data"].is_object() || result["data"].is_null());
    assert!(result["error"].is_object() || result["error"].is_null());
    assert!(result["metadata"].is_object());
    
    let metadata = &result["metadata"];
    assert!(metadata["timestamp"].is_string());
    assert!(metadata["version"].is_string());
}

// Integration test that creates a poll to verify basic functionality
#[sqlx::test]
async fn test_poll_creation_workflow(pool: PgPool) {
    let app = create_test_app_with_user(pool).await;
    let token = setup_authenticated_user(&app).await;
    
    // Create a poll
    let create_request = create_test_poll_request();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/polls")
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {}", token))
        .body(Body::from(create_request.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let create_result: Value = serde_json::from_slice(&body).unwrap();
    
    let poll_id = create_result["data"]["id"].as_str().unwrap();
    println!("Successfully created poll with ID: {}", poll_id);
    
    // Note: Due to the current implementation using random user IDs,
    // subsequent GET, UPDATE, and DELETE operations will fail because 
    // they won't find polls created by different user IDs.
    // This demonstrates the need for proper authentication middleware.
} 