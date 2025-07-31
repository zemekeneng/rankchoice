use axum::{routing::{get, post, put, delete}, Router};
use sqlx::PgPool;
use tower_http::cors::CorsLayer;
use uuid::Uuid;

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
        .layer(CorsLayer::permissive())
        .with_state(auth_service)
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION")
    }))
} 