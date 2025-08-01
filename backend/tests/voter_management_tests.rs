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
async fn test_create_voter_with_email(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Register a user and get their token
    let user_data = json!({
        "email": "testuser@example.com",
        "password": "testpassword123",
        "name": "Test User"
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
    let token = register_data["data"]["token"].as_str().unwrap();
    
    // Create a poll with this user
    let poll_data = json!({
        "title": "Test Poll",
        "description": "Test poll description",
        "pollType": "single_winner",
        "numWinners": 1,
        "candidates": [
            {"name": "Candidate A", "description": "First candidate"},
            {"name": "Candidate B", "description": "Second candidate"}
        ]
    });

    let poll_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/polls")
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(poll_data.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    let poll_body = to_bytes(poll_response.into_body(), usize::MAX).await.unwrap();
    let poll_result: Value = serde_json::from_slice(&poll_body).unwrap();
    let poll_id = poll_result["data"]["id"].as_str().unwrap();
    
    // Create voter with email
    let voter_request = json!({
        "email": "voter@example.com"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/polls/{}/invite", poll_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(voter_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    // Debug: print the response if test fails
    if result["success"] != true {
        println!("Error response: {}", serde_json::to_string_pretty(&result).unwrap());
    }
    
    assert_eq!(result["success"], true);
    assert!(result["data"]["id"].is_string());
    assert_eq!(result["data"]["email"], "voter@example.com");
    assert_eq!(result["data"]["pollId"], poll_id.to_string());
    assert!(result["data"]["ballotToken"].is_string());
    assert_eq!(result["data"]["hasVoted"], false);
    assert!(result["data"]["votingUrl"].as_str().unwrap().contains("/vote/"));
    assert!(result["data"]["invitedAt"].is_string());
    assert!(result["data"]["votedAt"].is_null());
}

#[sqlx::test]
async fn test_create_anonymous_voter(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Create a test poll
    let poll_id = create_test_poll(&pool).await;
    
    // Get auth token by registering and logging in a user
    let user_data = json!({
        "email": "testuser2@example.com",
        "password": "testpassword123",
        "name": "Test User"
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
    let token = register_data["data"]["token"].as_str().unwrap();
    
    // Create anonymous voter (no email)
    let voter_request = json!({});
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/polls/{}/invite", poll_id))
                .header("content-type", "application/json")
                .header("authorization", format!("Bearer {}", token))
                .body(Body::from(voter_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert!(result["data"]["id"].is_string());
    assert!(result["data"]["email"].as_str().unwrap().starts_with("Anonymous-"));
    assert_eq!(result["data"]["pollId"], poll_id.to_string());
    assert!(result["data"]["ballotToken"].is_string());
    assert_eq!(result["data"]["hasVoted"], false);
}

#[sqlx::test]
async fn test_list_voters_empty(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Create a test poll
    let poll_id = create_test_poll(&pool).await;
    
    // Get auth token by registering and logging in a user
    let user_data = json!({
        "email": "listuser@example.com",
        "password": "testpassword123",
        "name": "Test User"
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
    let token = register_data["data"]["token"].as_str().unwrap();
    
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/polls/{}/voters", poll_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["voters"].as_array().unwrap().len(), 0);
    assert_eq!(result["data"]["total"], 0);
    assert_eq!(result["data"]["votedCount"], 0);
    assert_eq!(result["data"]["pendingCount"], 0);
}

#[sqlx::test]
async fn test_list_voters_with_data(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Create a test poll
    let poll_id = create_test_poll(&pool).await;
    
    // Get auth token by registering and logging in a user
    let user_data = json!({
        "email": "listwithdata@example.com",
        "password": "testpassword123",
        "name": "Test User"
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
    let token = register_data["data"]["token"].as_str().unwrap();
    
    // Create some voters
    let voters_to_add = [
        json!({"email": "alice@example.com"}),
        json!({"email": "bob@example.com"}),
        json!({}), // Anonymous
    ];
    
    for voter_data in &voters_to_add {
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/polls/{}/invite", poll_id))
                    .header("content-type", "application/json")
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::from(voter_data.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
    }
    
    // List voters
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/polls/{}/voters", poll_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["voters"].as_array().unwrap().len(), 3);
    assert_eq!(result["data"]["total"], 3);
    assert_eq!(result["data"]["votedCount"], 0); // No votes cast yet
    assert_eq!(result["data"]["pendingCount"], 3);
    
    // Verify voter data structure
    let voters = result["data"]["voters"].as_array().unwrap();
    for voter in voters {
        assert!(voter["id"].is_string());
        assert_eq!(voter["pollId"], poll_id.to_string());
        assert!(voter["email"].is_string());
        assert!(voter["ballotToken"].is_string());
        assert_eq!(voter["hasVoted"], false);
        assert!(voter["invitedAt"].is_string());
        assert!(voter["votedAt"].is_null());
        assert!(voter["votingUrl"].as_str().unwrap().contains("/vote/"));
    }
}

#[sqlx::test]
async fn test_create_voter_unauthorized(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    let poll_id = create_test_poll(&pool).await;
    let voter_request = json!({
        "email": "voter@example.com"
    });
    
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/polls/{}/invite", poll_id))
                .header("content-type", "application/json")
                .body(Body::from(voter_request.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn test_list_voters_unauthorized(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    let poll_id = create_test_poll(&pool).await;
    
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/polls/{}/voters", poll_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}