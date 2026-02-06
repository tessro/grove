mod api;
mod db;
mod llm;
mod models;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use api::AppState;
use db::Db;
use llm::LlmClient;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    dotenvy::dotenv().ok();

    let api_key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY must be set");
    let model = std::env::var("GROVE_MODEL").ok();
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let db = Db::new("grove.db").expect("Failed to initialize database");
    let llm = LlmClient::new(api_key, model);

    let state = Arc::new(AppState { db, llm });

    let api_routes = Router::new()
        .route("/docs", post(api::create_doc))
        .route("/docs/{id}", get(api::get_doc))
        .route("/docs/{id}/chat", post(api::chat))
        .route("/docs/{id}/heartbeat", post(api::heartbeat))
        .route("/docs/{id}/messages", get(api::get_messages));

    let app = Router::new()
        .nest("/api", api_routes)
        .fallback_service(ServeDir::new("frontend/dist").fallback(get(spa_fallback)))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = format!("0.0.0.0:{port}");
    tracing::info!("Grove listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn spa_fallback() -> impl IntoResponse {
    match tokio::fs::read_to_string("frontend/dist/index.html").await {
        Ok(html) => Html(html).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}
