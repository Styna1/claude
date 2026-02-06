mod config;
mod db;
mod game;
mod protocol;
mod server;

use std::sync::Arc;
use axum::{routing::get, Router};
use tower_http::services::ServeDir;
use tracing_subscriber;

use crate::config::SERVER_PORT;
use crate::db::Database;
use crate::game::engine;
use crate::server::http;
use crate::server::ws;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Ensure data directory exists
    std::fs::create_dir_all("data").ok();

    // Initialize database
    let db = Arc::new(Database::new("data/game.db"));
    println!("✅ Database initialized");

    // Create game world
    let world = engine::create_world();
    println!("✅ Game world created ({}x{})", config::WORLD_SIZE, config::WORLD_SIZE);

    // Start game loop
    let world_clone = world.clone();
    tokio::spawn(async move {
        engine::game_loop(world_clone).await;
    });
    println!("✅ Game loop running at {} TPS", config::TICK_RATE);

    // WebSocket state
    let ws_state = ws::WsState {
        world: world.clone(),
        db: db.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/ws", get(ws::ws_handler).with_state(ws_state))
        .merge(http::api_routes(db))
        .fallback_service(ServeDir::new("static"));

    let addr = format!("0.0.0.0:{}", SERVER_PORT);
    println!("🎮 Agar.io clone running at http://localhost:{}", SERVER_PORT);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
