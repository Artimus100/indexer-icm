use redis::Client;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, RwLock};

pub type WebSocketSender = Arc<Mutex<futures::stream::SplitSink<axum::extract::ws::WebSocket, axum::extract::ws::Message>>>;

pub struct AppState {
    pub redis_client: Client,
    // Room-based subscriptions
    pub room_subscribers: Arc<RwLock<HashMap<String, Vec<WebSocketSender>>>>,
    // Legacy token-based subscriptions (keeping for backward compatibility)
    pub token_subscribers: Arc<RwLock<HashMap<String, Vec<WebSocketSender>>>>,
    pub all_tokens_subscribers: Arc<RwLock<Vec<WebSocketSender>>>,
}

impl AppState {
    pub fn new() -> Self {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://127.0.0.1".to_string());
        
        let redis_client = Client::open(redis_url.as_str())
            .expect("Failed to connect to Redis, check REDIS_URL environment variable");

        AppState {
            redis_client,
            room_subscribers: Arc::new(RwLock::new(HashMap::new())),
            token_subscribers: Arc::new(RwLock::new(HashMap::new())),
            all_tokens_subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

pub type SharedState = Arc<AppState>;
