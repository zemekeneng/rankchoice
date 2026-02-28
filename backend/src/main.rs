use axum::{
    routing::{get, post, put, delete},
    Router,
    Json,
};
use serde::Serialize;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;
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

async fn create_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in environment");

    tracing::info!("Connecting to database...");

    #[cfg(feature = "lambda")]
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(60))
        .connect(&database_url)
        .await?;

    #[cfg(not(feature = "lambda"))]
    let pool = PgPoolOptions::new()
        .max_connections(30)
        .min_connections(5)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

fn create_router(auth_service: AuthService) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/auth/register", post(api::auth::register))
        .route("/api/auth/login", post(api::auth::login))
        .route("/api/auth/refresh", post(api::auth::refresh))
        .route("/api/public/polls/:id", get(api::polls::get_public_poll))
        .route("/api/public/polls/:id/vote", post(api::voting::submit_anonymous_vote))
        .route("/api/polls", get(api::polls::list_polls))
        .route("/api/polls", post(api::polls::create_poll))
        .route("/api/polls/:id", get(api::polls::get_poll))
        .route("/api/polls/:id", put(api::polls::update_poll))
        .route("/api/polls/:id", delete(api::polls::delete_poll))
        .route("/api/polls/:id/candidates", get(api::candidates::list_candidates))
        .route("/api/polls/:id/candidates", post(api::candidates::add_candidate))
        .route("/api/polls/:id/candidates/order", put(api::candidates::reorder_candidates))
        .route("/api/candidates/:id", put(api::candidates::update_candidate))
        .route("/api/candidates/:id", delete(api::candidates::delete_candidate))
        .route("/api/polls/:id/invite", post(api::voters::create_voter))
        .route("/api/polls/:id/voters", get(api::voters::list_voters))
        .route("/api/polls/:id/registration", post(api::voters::create_registration_link))
        .route("/api/vote/:token", get(api::voting::get_ballot))
        .route("/api/vote/:token", post(api::voting::submit_ballot))
        .route("/api/vote/:token/receipt", get(api::voting::get_voting_receipt))
        .route("/api/polls/:id/results", get(api::results::get_poll_results))
        .route("/api/polls/:id/results/rounds", get(api::results::get_rcv_rounds))
        .route("/api/polls/:id/ballots/anonymous", get(api::results::get_anonymous_ballots))
        .layer(CorsLayer::permissive())
        .with_state(auth_service)
}

#[cfg(not(feature = "lambda"))]
#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    dotenv::dotenv().ok();

    let pool = create_pool().await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("Database migrations completed");

    let auth_service = AuthService::new(pool);
    let app = create_router(auth_service);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse()
        .expect("PORT must be a valid u16");
    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server running on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(feature = "lambda")]
#[tokio::main]
async fn main() -> Result<(), lambda_http::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .without_time()
        .init();

    let pool = create_pool().await.expect("Failed to create database pool");
    let auth_service = AuthService::new(pool);
    let app = create_router(auth_service);

    lambda_http::run(app).await
}
