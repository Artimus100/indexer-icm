use axum::{Router, routing::get};
use std::{net::SocketAddr, sync::Arc};
use tracing::{error, info};

use crate::{handlers::ws_handler, redis_consumer::RedisStreamConsumer, state::AppState};

mod handlers;
mod redis_consumer;
mod state;
mod types;
mod websocket;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file
    dotenvy::dotenv().ok();
    
    tracing_subscriber::fmt::init();

    let state = Arc::new(AppState::new());

    // Start Redis stream consumer in background
    let redis_client = state.redis_client.clone();
    let consumer_state = state.clone();
    tokio::spawn(async move {
        let mut consumer = RedisStreamConsumer::new(redis_client, consumer_state)
            .await
            .expect("Failed to create Redis consumer");
        if let Err(e) = consumer.run().await {
            error!("Redis consumer error: {}", e);
        }
    });

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8001));
    info!("WebSocket server listening on ws://{}", addr);
    info!("Redis stream consumer started");

    axum::serve(
        tokio::net::TcpListener::bind(addr).await.unwrap(),
        app.into_make_service(),
    )
    .await
    .unwrap();
}
