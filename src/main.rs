mod handlers;
mod models;

use axum::{
    routing::{delete, get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize database pool
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    tracing::info!("Migrations completed successfully");

    // Build router
    let app = Router::new()
        // Frontend routes
        .route("/", get(handlers::pages::game_list))
        .route("/game/{id}", get(handlers::pages::game_detail))
        // API routes
        .route("/api/games", get(handlers::api::list_games).post(handlers::api::create_game))
        .route("/api/games/{id}", get(handlers::api::get_game).delete(handlers::api::delete_game))
        .route("/api/games/{id}/move", post(handlers::api::make_move))
        // Static files
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    tracing::info!("Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
