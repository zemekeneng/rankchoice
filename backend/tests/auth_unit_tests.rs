use sqlx::PgPool;
use uuid::Uuid;

use rankchoice_api::{
    models::user::{CreateUserRequest, LoginRequest, User},
    services::auth::{AuthError, AuthService},
};

#[sqlx::test]
async fn test_password_hashing(pool: PgPool) {
    let auth_service = AuthService::new(pool);
    
    let password = "test_password_123";
    let hash1 = auth_service.hash_password(password).unwrap();
    let hash2 = auth_service.hash_password(password).unwrap();
    
    // Hashes should be different (due to salt)
    assert_ne!(hash1, hash2);
    
    // Both should verify correctly
    assert!(auth_service.verify_password(password, &hash1).unwrap());
    assert!(auth_service.verify_password(password, &hash2).unwrap());
    
    // Wrong password should fail
    assert!(!auth_service.verify_password("wrong_password", &hash1).unwrap());
}

#[sqlx::test]
async fn test_jwt_token_generation_and_verification(pool: PgPool) {
    let auth_service = AuthService::new(pool);
    
    let user = User {
        id: Uuid::new_v4(),
        email: "test@example.com".to_string(),
        password_hash: "hash".to_string(),
        name: Some("Test User".to_string()),
        role: "pollster".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    
    // Generate tokens
    let access_token = auth_service.generate_token(&user, false).unwrap();
    let refresh_token = auth_service.generate_token(&user, true).unwrap();
    
    // Tokens should be different
    assert_ne!(access_token, refresh_token);
    
    // Verify tokens
    let access_claims = auth_service.verify_token(&access_token).unwrap();
    let refresh_claims = auth_service.verify_token(&refresh_token).unwrap();
    
    // Claims should match user data
    assert_eq!(access_claims.sub, user.id.to_string());
    assert_eq!(access_claims.email, user.email);
    assert_eq!(access_claims.role, user.role);
    
    assert_eq!(refresh_claims.sub, user.id.to_string());
    assert_eq!(refresh_claims.email, user.email);
    assert_eq!(refresh_claims.role, user.role);
    
    // Access token should expire sooner than refresh token
    assert!(access_claims.exp < refresh_claims.exp);
}

#[sqlx::test]
async fn test_invalid_jwt_token(pool: PgPool) {
    let auth_service = AuthService::new(pool);
    
    // Test completely invalid token
    let result = auth_service.verify_token("invalid.token.here");
    assert!(result.is_err());
    
    // Test malformed token
    let result = auth_service.verify_token("not.a.jwt");
    assert!(result.is_err());
    
    // Test empty token
    let result = auth_service.verify_token("");
    assert!(result.is_err());
}

#[sqlx::test]
async fn test_user_registration_service(pool: PgPool) {
    let auth_service = AuthService::new(pool.clone());
    
    let request = CreateUserRequest {
        email: "service_test@example.com".to_string(),
        password: "password123".to_string(),
        name: Some("Service Test".to_string()),
    };
    
    let result = auth_service.register(request).await.unwrap();
    
    // Verify response structure
    assert_eq!(result.user.email, "service_test@example.com");
    assert_eq!(result.user.name, Some("Service Test".to_string()));
    assert_eq!(result.user.role, "pollster");
    assert!(!result.token.is_empty());
    assert!(!result.refresh_token.is_empty());
    
    // Verify user exists in database
    let user = User::find_by_email(&pool, "service_test@example.com")
        .await
        .unwrap()
        .unwrap();
    
    assert_eq!(user.email, "service_test@example.com");
    assert_eq!(user.name, Some("Service Test".to_string()));
    
    // Verify password was hashed
    assert_ne!(user.password_hash, "password123");
    assert!(auth_service.verify_password("password123", &user.password_hash).unwrap());
}

#[sqlx::test]
async fn test_user_login_service(pool: PgPool) {
    let auth_service = AuthService::new(pool.clone());
    
    // First register a user
    let register_request = CreateUserRequest {
        email: "login_test@example.com".to_string(),
        password: "password123".to_string(),
        name: Some("Login Test".to_string()),
    };
    
    auth_service.register(register_request).await.unwrap();
    
    // Now test login
    let login_request = LoginRequest {
        email: "login_test@example.com".to_string(),
        password: "password123".to_string(),
    };
    
    let result = auth_service.login(login_request).await.unwrap();
    
    // Verify response
    assert_eq!(result.user.email, "login_test@example.com");
    assert!(!result.token.is_empty());
    assert!(!result.refresh_token.is_empty());
    
    // Verify token is valid
    let claims = auth_service.verify_token(&result.token).unwrap();
    assert_eq!(claims.email, "login_test@example.com");
}

#[sqlx::test]
async fn test_duplicate_user_registration(pool: PgPool) {
    let auth_service = AuthService::new(pool);
    
    let request1 = CreateUserRequest {
        email: "duplicate@example.com".to_string(),
        password: "password123".to_string(),
        name: Some("First User".to_string()),
    };
    
    let request2 = CreateUserRequest {
        email: "duplicate@example.com".to_string(),
        password: "different_password".to_string(),
        name: Some("Second User".to_string()),
    };
    
    // First registration should succeed
    auth_service.register(request1).await.unwrap();
    
    // Second registration should fail
    let result = auth_service.register(request2).await;
    assert!(matches!(result, Err(AuthError::UserAlreadyExists)));
}

#[sqlx::test]
async fn test_login_invalid_credentials(pool: PgPool) {
    let auth_service = AuthService::new(pool);
    
    // Register a user
    let register_request = CreateUserRequest {
        email: "invalid_creds@example.com".to_string(),
        password: "correct_password".to_string(),
        name: Some("Invalid Creds Test".to_string()),
    };
    
    auth_service.register(register_request).await.unwrap();
    
    // Test with wrong password
    let login_request = LoginRequest {
        email: "invalid_creds@example.com".to_string(),
        password: "wrong_password".to_string(),
    };
    
    let result = auth_service.login(login_request).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
    
    // Test with non-existent user
    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "any_password".to_string(),
    };
    
    let result = auth_service.login(login_request).await;
    assert!(matches!(result, Err(AuthError::InvalidCredentials)));
}

#[sqlx::test]
async fn test_refresh_token_service(pool: PgPool) {
    let auth_service = AuthService::new(pool.clone());
    
    // Register a user and get tokens
    let register_request = CreateUserRequest {
        email: "refresh_service@example.com".to_string(),
        password: "password123".to_string(),
        name: Some("Refresh Service Test".to_string()),
    };
    
    let auth_response = auth_service.register(register_request).await.unwrap();
    let original_token = auth_response.token.clone();
    let refresh_token = auth_response.refresh_token;
    
    // Wait a moment to ensure different timestamp
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    
    // Use refresh token to get new access token
    let new_token = auth_service.refresh_token(&refresh_token).await.unwrap();
    
    // New token should be different
    assert_ne!(original_token, new_token);
    
    // Both tokens should be valid
    let original_claims = auth_service.verify_token(&original_token).unwrap();
    let new_claims = auth_service.verify_token(&new_token).unwrap();
    
    // Claims should have same user data but different timestamps
    assert_eq!(original_claims.sub, new_claims.sub);
    assert_eq!(original_claims.email, new_claims.email);
    assert_eq!(original_claims.role, new_claims.role);
    assert!(original_claims.iat < new_claims.iat);
}

#[sqlx::test]
async fn test_refresh_token_invalid(pool: PgPool) {
    let auth_service = AuthService::new(pool);
    
    // Test with invalid refresh token
    let result = auth_service.refresh_token("invalid.refresh.token").await;
    assert!(result.is_err());
    
    // Test with empty token
    let result = auth_service.refresh_token("").await;
    assert!(result.is_err());
} 