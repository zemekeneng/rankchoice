use axum::{routing::{get, post, put, delete}, Router};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use uuid::Uuid;
use serde_json::json;

use rankchoice_api::services::auth::AuthService;

// Consistent test user ID for all tests
pub const TEST_USER_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

pub async fn create_test_user(pool: &PgPool) -> Uuid {
    let user_id = Uuid::parse_str(TEST_USER_ID).unwrap();
    
    // Try to insert test user, ignore if already exists
    let _ = sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, name, role)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (id) DO NOTHING
        "#,
        user_id,
        "test@example.com",
        "$argon2id$v=19$m=4096,t=3,p=1$salt$hashedpassword", // dummy hash
        "Test User",
        "pollster"
    )
    .execute(pool)
    .await;
    
    user_id
}

pub async fn create_test_app_with_user(pool: PgPool) -> Router {
    // Create test user for polls tests
    create_test_user(&pool).await;
    create_test_app(pool).await
}

pub async fn create_test_app(pool: PgPool) -> Router {
    // Initialize services
    let auth_service = AuthService::new(pool.clone());

    // Build test app with same routes as main app
    Router::new()
        .route("/health", get(health_handler))
        // Authentication routes (public)
        .route("/api/auth/register", post(rankchoice_api::api::auth::register))
        .route("/api/auth/login", post(rankchoice_api::api::auth::login))
        .route("/api/auth/refresh", post(rankchoice_api::api::auth::refresh))
        // Protected poll routes
        .route("/api/polls", get(rankchoice_api::api::polls::list_polls))
        .route("/api/polls", post(rankchoice_api::api::polls::create_poll))
        .route("/api/polls/:id", get(rankchoice_api::api::polls::get_poll))
        .route("/api/polls/:id", put(rankchoice_api::api::polls::update_poll))
        .route("/api/polls/:id", delete(rankchoice_api::api::polls::delete_poll))
        // Candidate management routes
        .route("/api/polls/:id/candidates", get(rankchoice_api::api::candidates::list_candidates))
        .route("/api/polls/:id/candidates", post(rankchoice_api::api::candidates::add_candidate))
        .route("/api/polls/:id/candidates/order", put(rankchoice_api::api::candidates::reorder_candidates))
        .route("/api/candidates/:id", put(rankchoice_api::api::candidates::update_candidate))
        .route("/api/candidates/:id", delete(rankchoice_api::api::candidates::delete_candidate))
        // Voter management routes
        .route("/api/polls/:id/invite", post(rankchoice_api::api::voters::create_voter))
        .route("/api/polls/:id/voters", get(rankchoice_api::api::voters::list_voters))
        .route("/api/polls/:id/registration", post(rankchoice_api::api::voters::create_registration_link))
        // Voting routes (public)
        .route("/api/vote/:token", get(rankchoice_api::api::voting::get_ballot))
        .route("/api/vote/:token", post(rankchoice_api::api::voting::submit_ballot))
        .route("/api/vote/:token/receipt", get(rankchoice_api::api::voting::get_voting_receipt))
        // Results routes (protected)
        .route("/api/polls/:id/results", get(rankchoice_api::api::results::get_poll_results))
        .route("/api/polls/:id/results/rounds", get(rankchoice_api::api::results::get_rcv_rounds))
        .layer(CorsLayer::permissive())
        .with_state(auth_service)
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
}

// Test helper functions
pub async fn setup_test_user(pool: &PgPool) -> Uuid {
    create_test_user(pool).await
}

pub async fn create_test_poll(pool: &PgPool) -> Uuid {
    let user_id = create_test_user(pool).await;
    
    let poll_id = sqlx::query!(
        r#"
        INSERT INTO polls (user_id, title, description, poll_type, num_winners, is_public, registration_required)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
        user_id,
        "Test Poll",
        "Test poll description",
        "single_winner",
        1,
        false,
        false
    )
    .fetch_one(pool)
    .await
    .unwrap()
    .id;
    
    poll_id
}

pub async fn create_test_candidates(pool: &PgPool, poll_id: Uuid) -> Vec<Uuid> {
    let candidates = vec![
        ("Candidate A", "Description A"),
        ("Candidate B", "Description B"),
        ("Candidate C", "Description C"),
    ];
    
    let mut candidate_ids = Vec::new();
    
    for (i, (name, description)) in candidates.iter().enumerate() {
        let candidate_id = sqlx::query!(
            r#"
            INSERT INTO candidates (poll_id, name, description, display_order)
            VALUES ($1, $2, $3, $4)
            RETURNING id
            "#,
            poll_id,
            name,
            description,
            i as i32 + 1
        )
        .fetch_one(pool)
        .await
        .unwrap()
        .id;
        
        candidate_ids.push(candidate_id);
    }
    
    candidate_ids
} 