use axum::{
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use rankchoice_api::models::ballot::{Ballot, BallotRanking, Voter};

mod common;
use common::*;

#[sqlx::test]
async fn test_get_poll_results_not_found(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let non_existent_poll_id = Uuid::new_v4();
    
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/polls/{}/results", non_existent_poll_id))
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
async fn test_get_rcv_rounds_not_found(pool: PgPool) {
    let app = create_test_app(pool).await;
    
    let non_existent_poll_id = Uuid::new_v4();
    
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/polls/{}/results/rounds", non_existent_poll_id))
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
async fn test_poll_results_no_votes(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Setup test poll without any votes
    setup_test_user(&pool).await;
    let poll_id = create_test_poll(&pool).await;
    create_test_candidates(&pool, poll_id).await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/polls/{}/results", poll_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["poll_id"], poll_id.to_string());
    assert_eq!(result["data"]["total_votes"], 0);
    assert_eq!(result["data"]["status"], "no_votes");
    assert!(result["data"]["winner"].is_null());
}

#[sqlx::test]
async fn test_rcv_rounds_no_votes(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Setup test poll without any votes
    setup_test_user(&pool).await;
    let poll_id = create_test_poll(&pool).await;
    create_test_candidates(&pool, poll_id).await;
    
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/polls/{}/results/rounds", poll_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["total_ballots"], 0);
    assert_eq!(result["data"]["exhausted_ballots"], 0);
    assert_eq!(result["data"]["rounds"].as_array().unwrap().len(), 0);
}

#[sqlx::test]
async fn test_results_with_votes(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Setup test poll with votes
    setup_test_user(&pool).await;
    let poll_id = create_test_poll(&pool).await;
    let candidate_ids = create_test_candidates(&pool, poll_id).await;
    
    // Create a voter and submit a ballot
    let voter = Voter::create(
        &pool, 
        poll_id, 
        Some("voter@example.com".to_string()), 
        None, 
        None
    ).await.expect("Failed to create voter");
    
    let rankings = vec![
        BallotRanking {
            candidate_id: candidate_ids[0],
            rank: 1,
        },
        BallotRanking {
            candidate_id: candidate_ids[1],
            rank: 2,
        },
    ];
    
    Ballot::create(&pool, voter.id, poll_id, rankings, None)
        .await
        .expect("Failed to create ballot");
    
    // Test getting results
    let request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/polls/{}/results", poll_id))
        .body(Body::empty())
        .unwrap();

    let response = app.clone().oneshot(request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["poll_id"], poll_id.to_string());
    assert_eq!(result["data"]["total_votes"], 1);
    assert!(result["data"]["winner"].is_object());
    
    // Test getting RCV rounds
    let rounds_request = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/polls/{}/results/rounds", poll_id))
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(rounds_request).await.unwrap();
    
    assert_eq!(response.status(), StatusCode::OK);
    
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let result: Value = serde_json::from_slice(&body).unwrap();
    
    assert_eq!(result["success"], true);
    assert_eq!(result["data"]["total_ballots"], 1);
    assert!(result["data"]["rounds"].is_array());
    
    let rounds = result["data"]["rounds"].as_array().unwrap();
    assert!(!rounds.is_empty());
} 