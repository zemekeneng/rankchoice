use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;

mod common;
use common::*;

/// Test the complete voter management workflow
#[sqlx::test]
async fn test_complete_voter_management_workflow(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Step 1: Register a user and get auth token
    let user_data = json!({
        "email": "pollcreator@example.com",
        "password": "testpassword123",
        "name": "Poll Creator"
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
    
    // Step 2: Create a poll
    let poll_data = json!({
        "title": "Complete Test Poll",
        "description": "Testing voter management",
        "pollType": "single_winner",
        "numWinners": 1,
        "candidates": [
            {"name": "Alice", "description": "Candidate A"},
            {"name": "Bob", "description": "Candidate B"},
            {"name": "Charlie", "description": "Candidate C"}
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

    // Step 3: Verify empty voters list
    let voters_response = app
        .clone()
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

    assert_eq!(voters_response.status(), StatusCode::OK);
    let voters_body = to_bytes(voters_response.into_body(), usize::MAX).await.unwrap();
    let voters_result: Value = serde_json::from_slice(&voters_body).unwrap();
    
    assert_eq!(voters_result["success"], true);
    assert_eq!(voters_result["data"]["total"], 0);
    assert_eq!(voters_result["data"]["votedCount"], 0);
    assert_eq!(voters_result["data"]["pendingCount"], 0);

    // Step 4: Create a registration link
    let registration_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/polls/{}/registration", poll_id))
                .header("authorization", format!("Bearer {}", token))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(registration_response.status(), StatusCode::OK);
    let reg_body = to_bytes(registration_response.into_body(), usize::MAX).await.unwrap();
    let reg_result: Value = serde_json::from_slice(&reg_body).unwrap();
    
    assert_eq!(reg_result["success"], true);
    assert_eq!(reg_result["data"]["pollId"], poll_id);
    assert!(reg_result["data"]["registrationToken"].as_str().unwrap().starts_with("reg_"));
    assert!(reg_result["data"]["registrationUrl"].as_str().unwrap().contains("/register/"));
    assert!(reg_result["data"]["createdAt"].is_string());

    // Step 5: Add voters via invite
    let voters_to_add = [
        json!({"email": "voter1@example.com"}),
        json!({"email": "voter2@example.com"}),
        json!({}), // Anonymous voter
        json!({"email": "voter3@example.com"}),
    ];

    let mut voter_tokens = Vec::new();
    
    for (i, voter_data) in voters_to_add.iter().enumerate() {
        let invite_response = app
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

        assert_eq!(invite_response.status(), StatusCode::OK);
        
        let invite_body = to_bytes(invite_response.into_body(), usize::MAX).await.unwrap();
        let invite_result: Value = serde_json::from_slice(&invite_body).unwrap();
        
        assert_eq!(invite_result["success"], true);
        assert!(invite_result["data"]["id"].is_string());
        assert_eq!(invite_result["data"]["pollId"], poll_id);
        assert!(invite_result["data"]["ballotToken"].is_string());
        assert_eq!(invite_result["data"]["hasVoted"], false);
        assert!(invite_result["data"]["votingUrl"].as_str().unwrap().contains("/vote/"));
        
        // Verify email format
        if i == 2 {
            // Anonymous voter
            assert!(invite_result["data"]["email"].as_str().unwrap().starts_with("Anonymous-"));
        } else if i < 2 {
            // Named voters (voter1, voter2)
            let expected_email = format!("voter{}@example.com", i + 1);
            assert_eq!(invite_result["data"]["email"], expected_email);
        } else {
            // voter3 (i == 3, but it's the 4th voter after anonymous)
            assert_eq!(invite_result["data"]["email"], "voter3@example.com");
        }
        
        voter_tokens.push(invite_result["data"]["ballotToken"].as_str().unwrap().to_string());
    }

    // Step 6: Verify voters list with data
    let final_voters_response = app
        .clone()
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

    assert_eq!(final_voters_response.status(), StatusCode::OK);
    let final_voters_body = to_bytes(final_voters_response.into_body(), usize::MAX).await.unwrap();
    let final_voters_result: Value = serde_json::from_slice(&final_voters_body).unwrap();
    
    assert_eq!(final_voters_result["success"], true);
    assert_eq!(final_voters_result["data"]["total"], 4);
    assert_eq!(final_voters_result["data"]["votedCount"], 0); // No votes cast yet
    assert_eq!(final_voters_result["data"]["pendingCount"], 4);
    
    // Verify voter data structure
    let voters = final_voters_result["data"]["voters"].as_array().unwrap();
    assert_eq!(voters.len(), 4);
    
    for voter in voters {
        assert!(voter["id"].is_string());
        assert_eq!(voter["pollId"], poll_id);
        assert!(voter["email"].is_string());
        assert!(voter["ballotToken"].is_string());
        assert_eq!(voter["hasVoted"], false);
        assert!(voter["invitedAt"].is_string());
        assert!(voter["votedAt"].is_null());
        assert!(voter["votingUrl"].as_str().unwrap().contains("/vote/"));
    }

    // Step 7: Test voter tokens work for voting (optional verification)
    for token in voter_tokens.iter() {
        let ballot_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(&format!("/api/vote/{}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(ballot_response.status(), StatusCode::OK);
        
        let ballot_body = to_bytes(ballot_response.into_body(), usize::MAX).await.unwrap();
        let ballot_result: Value = serde_json::from_slice(&ballot_body).unwrap();
        
        assert_eq!(ballot_result["success"], true);
        assert_eq!(ballot_result["data"]["poll"]["id"], poll_id);
        assert_eq!(ballot_result["data"]["poll"]["candidates"].as_array().unwrap().len(), 3);
    }
}

#[sqlx::test]
async fn test_voter_management_error_cases(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    let fake_poll_id = "550e8400-e29b-41d4-a716-446655440001";
    
    // Test unauthorized access
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/polls/{}/invite", fake_poll_id))
                .header("content-type", "application/json")
                .body(Body::from(json!({"email": "test@example.com"}).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test list voters unauthorized
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(&format!("/api/polls/{}/voters", fake_poll_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Test registration link unauthorized
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(&format!("/api/polls/{}/registration", fake_poll_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test]
async fn test_registration_link_functionality(pool: PgPool) {
    let app = create_test_app(pool.clone()).await;
    
    // Register user and create poll
    let user_data = json!({
        "email": "registration@example.com",
        "password": "testpassword123",
        "name": "Registration Test User"
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
    
    // Create a poll
    let poll_data = json!({
        "title": "Registration Test Poll",
        "description": "Testing registration links",
        "pollType": "single_winner",
        "numWinners": 1,
        "candidates": [
            {"name": "Option A", "description": "First option"},
            {"name": "Option B", "description": "Second option"}
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

    // Create multiple registration links
    for i in 0..3 {
        let reg_response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(&format!("/api/polls/{}/registration", poll_id))
                    .header("authorization", format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(reg_response.status(), StatusCode::OK);
        
        let reg_body = to_bytes(reg_response.into_body(), usize::MAX).await.unwrap();
        let reg_result: Value = serde_json::from_slice(&reg_body).unwrap();
        
        assert_eq!(reg_result["success"], true);
        assert_eq!(reg_result["data"]["pollId"], poll_id);
        
        let token = reg_result["data"]["registrationToken"].as_str().unwrap();
        assert!(token.starts_with("reg_"));
        assert!(token.len() > 30); // Should be a UUID-based token (reg_ + UUID = ~40 chars)
        
        let url = reg_result["data"]["registrationUrl"].as_str().unwrap();
        assert!(url.contains(&format!("/register/{}", token)));
        
        assert!(reg_result["data"]["createdAt"].is_string());
        
        println!("Registration link {}: {}", i + 1, url);
    }
}