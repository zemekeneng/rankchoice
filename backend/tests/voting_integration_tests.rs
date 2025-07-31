use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use rankchoice_api::models::ballot::Voter;

mod common;
use common::*;

#[sqlx::test]
async fn test_get_ballot_invalid_token(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/vote/invalid-token")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "NOT_FOUND");
}

#[sqlx::test]
async fn test_submit_ballot_invalid_token(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let ballot_data = json!({
        "rankings": [
            {"candidate_id": Uuid::new_v4(), "rank": 1}
        ]
    });
    
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/vote/invalid-token")
        .header("content-type", "application/json")
        .body(Body::from(ballot_data.to_string()))
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "NOT_FOUND");
}

#[sqlx::test]
async fn test_voting_receipt_invalid_token(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/vote/invalid-token/receipt")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], false);
    assert_eq!(result["error"]["code"], "NOT_FOUND");
}

#[sqlx::test]
async fn test_voting_workflow_with_valid_voter(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Setup test poll and candidates
    setup_test_user(&pool).await;
    let poll_id = create_test_poll(&pool).await;
    let candidate_ids = create_test_candidates(&pool, poll_id).await;
    
    // Create a voter for the poll
    let voter = Voter::create(
        &pool, 
        poll_id, 
        Some("voter@example.com".to_string()), 
        None, 
        None
    ).await.expect("Failed to create voter");
    
    // Test getting ballot
    let get_ballot_request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/vote/{}", voter.ballot_token))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(get_ballot_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert!(result["data"]["poll"]["id"].is_string());
    assert_eq!(result["data"]["voter"]["has_voted"], false);
    
    // Test submitting ballot
    let ballot_data = json!({
        "rankings": [
            {"candidate_id": candidate_ids[0], "rank": 1},
            {"candidate_id": candidate_ids[1], "rank": 2}
        ]
    });
    
    let submit_request = Request::builder()
        .method(Method::POST)
        .uri(format!("/api/vote/{}", voter.ballot_token))
        .header("content-type", "application/json")
        .body(Body::from(ballot_data.to_string()))
        .unwrap();

    let response = app.clone().oneshot(submit_request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert!(result["data"]["ballot"]["id"].is_string());
    assert!(result["data"]["receipt"]["receipt_code"].is_string());
} 