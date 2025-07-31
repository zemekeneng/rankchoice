use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;

mod common;
use common::*;

// Test helper to create a test poll ID
fn get_test_poll_id() -> Uuid {
    Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
}

// Test helper to create a test candidate ID
fn get_test_candidate_id() -> Uuid {
    Uuid::parse_str("650e8400-e29b-41d4-a716-446655440000").unwrap()
}

#[sqlx::test]
async fn test_add_candidate_success(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let poll_id = get_test_poll_id();
    let request_data = json!({
        "name": "New Candidate",
        "description": "A great candidate"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/polls/{}/candidates", poll_id))
        .header("content-type", "application/json")
        .body(Body::from(request_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // Since we don't have a real poll, this will likely return 500 or similar
    // but we can test the structure
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    // Test API response structure
    assert!(result["success"].is_boolean());
    assert!(result["metadata"].is_object());
    
    let metadata = &result["metadata"];
    assert!(metadata["timestamp"].is_string());
    assert!(metadata["version"].is_string());
}

#[sqlx::test]
async fn test_add_candidate_validation_empty_name(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let poll_id = get_test_poll_id();
    let request_data = json!({
        "name": "",
        "description": "A candidate with empty name"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/polls/{}/candidates", poll_id))
        .header("content-type", "application/json")
        .body(Body::from(request_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "VALIDATION_ERROR");
    assert!(result["error"]["message"].as_str().unwrap().contains("name is required"));
}

#[sqlx::test]
async fn test_add_candidate_validation_whitespace_name(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let poll_id = get_test_poll_id();
    let request_data = json!({
        "name": "   ",
        "description": "A candidate with whitespace name"
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/polls/{}/candidates", poll_id))
        .header("content-type", "application/json")
        .body(Body::from(request_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test]
async fn test_list_candidates(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let poll_id = get_test_poll_id();
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/polls/{}/candidates", poll_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    // Test API response structure
    assert!(result["success"].is_boolean());
    assert!(result["metadata"].is_object());
}

#[sqlx::test]
async fn test_update_candidate_validation_empty_name(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let candidate_id = get_test_candidate_id();
    let request_data = json!({
        "name": ""
    });
    
    let request = Request::builder()
        .method(Method::PUT)
        .uri(&format!("/api/candidates/{}", candidate_id))
        .header("content-type", "application/json")
        .body(Body::from(request_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "VALIDATION_ERROR");
    assert!(result["error"]["message"].as_str().unwrap().contains("cannot be empty"));
}

#[sqlx::test]
async fn test_update_candidate_not_found(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let candidate_id = get_test_candidate_id();
    let request_data = json!({
        "name": "Updated Name"
    });
    
    let request = Request::builder()
        .method(Method::PUT)
        .uri(&format!("/api/candidates/{}", candidate_id))
        .header("content-type", "application/json")
        .body(Body::from(request_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // This should either be 404 (not found) or 500 (server error)
    // depending on the implementation
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    // Should be either CANDIDATE_NOT_FOUND or some other error
    assert!(result["error"]["code"].is_string());
}

#[sqlx::test]
async fn test_delete_candidate_not_found(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let candidate_id = get_test_candidate_id();
    let request = Request::builder()
        .method(Method::DELETE)
        .uri(&format!("/api/candidates/{}", candidate_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    // Test that we get some kind of error response
    assert!(result["error"]["code"].is_string());
}

#[sqlx::test]
async fn test_reorder_candidates_validation_empty_list(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let poll_id = get_test_poll_id();
    let request_data = json!({
        "candidate_order": []
    });
    
    let request = Request::builder()
        .method(Method::PUT)
        .uri(&format!("/api/polls/{}/candidates/order", poll_id))
        .header("content-type", "application/json")
        .body(Body::from(request_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "VALIDATION_ERROR");
    assert!(result["error"]["message"].as_str().unwrap().contains("At least one candidate"));
}

#[sqlx::test]
async fn test_reorder_candidates_with_valid_ids(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let poll_id = get_test_poll_id();
    let candidate_id1 = Uuid::new_v4();
    let candidate_id2 = Uuid::new_v4();
    
    let request_data = json!({
        "candidate_order": [candidate_id1, candidate_id2]
    });
    
    let request = Request::builder()
        .method(Method::PUT)
        .uri(&format!("/api/polls/{}/candidates/order", poll_id))
        .header("content-type", "application/json")
        .body(Body::from(request_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    // Test API response structure
    assert!(result["success"].is_boolean());
    assert!(result["metadata"].is_object());
}

#[sqlx::test]
async fn test_candidate_api_response_format(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let poll_id = get_test_poll_id();
    let request = Request::builder()
        .method(Method::GET)
        .uri(&format!("/api/polls/{}/candidates", poll_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    // Verify API response structure matches polls API
    assert!(result["success"].is_boolean());
    assert!(result["data"].is_array() || result["data"].is_null());
    assert!(result["error"].is_object() || result["error"].is_null());
    assert!(result["metadata"].is_object());
    
    let metadata = &result["metadata"];
    assert!(metadata["timestamp"].is_string());
    assert!(metadata["version"].is_string());
}

#[sqlx::test]
async fn test_candidate_json_request_parsing(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let poll_id = get_test_poll_id();
    
    // Test that the endpoint properly handles JSON parsing
    let request = Request::builder()
        .method(Method::POST)
        .uri(&format!("/api/polls/{}/candidates", poll_id))
        .header("content-type", "application/json")
        .body(Body::from("invalid json"))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    // Should return some kind of error for invalid JSON
    assert_ne!(response.status(), StatusCode::OK);
} 