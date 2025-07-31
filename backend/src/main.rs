use axum::{
    routing::{get, post, put, delete},
    Router,
    Json,
};
use serde::Serialize;
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

mod api;
mod middleware;
mod models;
mod services;

use services::auth::AuthService;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenv::dotenv().ok();

    // Database connection
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment");
    
    tracing::info!("Connecting to database...");
    let pool = PgPool::connect(&database_url).await?;
    
    // Run database migrations
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations completed");

    // Initialize services
    let auth_service = AuthService::new(pool.clone());

    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health))
        // Authentication routes (public)
        .route("/api/auth/register", post(api::auth::register))
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/refresh", post(api::auth::refresh))
        // Protected poll routes
        .route("/api/polls", get(api::polls::list_polls))
        .route("/api/polls", post(api::polls::create_poll))
        .route("/api/polls/:id", get(api::polls::get_poll))
        .route("/api/polls/:id", put(api::polls::update_poll))
        .route("/api/polls/:id", delete(api::polls::delete_poll))
        // Candidate management routes
        .route("/api/polls/:id/candidates", get(api::candidates::list_candidates))
        .route("/api/polls/:id/candidates", post(api::candidates::add_candidate))
        .route("/api/polls/:id/candidates/order", put(api::candidates::reorder_candidates))
        .route("/api/candidates/:id", put(api::candidates::update_candidate))
        .route("/api/candidates/:id", delete(api::candidates::delete_candidate))
        // Voting routes (public)
        .route("/api/vote/:token", get(api::voting::get_ballot))
        .route("/api/vote/:token", post(api::voting::submit_ballot))
        .route("/api/vote/:token/receipt", get(api::voting::get_voting_receipt))
        // Results routes (protected)
        .route("/api/polls/:id/results", get(api::results::get_poll_results))
        .route("/api/polls/:id/results/rounds", get(api::results::get_rcv_rounds))
        .layer(CorsLayer::permissive())
        .with_state(auth_service);

    // Run our app with hyper
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("🚀 Server running on http://{}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
