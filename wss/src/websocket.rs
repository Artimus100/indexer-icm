use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::state::{SharedState, WebSocketSender};
use crate::types::{ClientRequest, ServerMessage};

pub async fn handle_socket(socket: WebSocket, state: SharedState) {
    let (websocket_sender, mut websocket_receiver) = socket.split();
    let shared_websocket_sender = Arc::new(Mutex::new(websocket_sender));

    info!("New WebSocket connection established");

    while let Some(Ok(msg)) = websocket_receiver.next().await {
        match msg {
            Message::Text(text) => match serde_json::from_str::<ClientRequest>(&text) {
                Ok(request) => match request {
                    ClientRequest::Subscribe { room, data } => {
                        info!("Client subscribing to room: {}", room);
                        handle_subscribe_room(
                            room,
                            data,
                            shared_websocket_sender.clone(),
                            &state,
                        )
                        .await;
                    }
                    ClientRequest::Unsubscribe { room } => {
                        info!("Client unsubscribing from room: {}", room);
                        handle_unsubscribe_room(room, &state).await;
                    }
                },
                Err(e) => {
                    error!("Failed to parse client request: {}", e);
                    let error_msg = ServerMessage::Error {
                        message: format!("Invalid request format: {}", e),
                    };
                    if let Ok(json) = serde_json::to_string(&error_msg) {
                        let _ = shared_websocket_sender
                            .lock()
                            .await
                            .send(Message::Text(json.into()))
                            .await;
                    }
                }
            },
            Message::Close(_) => {
                info!("WebSocket connection closed");
                break;
            }
            Message::Ping(_) => {
                // Handle ping if needed
            }
            Message::Pong(_) => {
                // Handle pong if needed
            }
            _ => {
                // Handle other message types
            }
        }
    }

    info!("WebSocket connection ended");
}

async fn handle_subscribe_room(
    room: String,
    _data: Option<Vec<String>>,
    sender: WebSocketSender,
    state: &SharedState,
) {
    // Add subscriber to the room's subscriber list
    {
        let mut subscribers = state.room_subscribers.write().await;
        subscribers
            .entry(room.clone())
            .or_insert_with(Vec::new)
            .push(sender.clone());
    }

    // Send confirmation message
    let confirmation = ServerMessage::SubscriptionConfirmed {
        room: room.clone(),
    };
    if let Ok(json) = serde_json::to_string(&confirmation) {
        let _ = sender.lock().await.send(Message::Text(json.into())).await;
    }

    info!("Client subscribed to room: {}", room);
    
    // Handle specific room types
    if room == "coins" {
        info!("Client subscribed to launch coin events");
    } else if room.starts_with("trades@") {
        let pool_id = room.strip_prefix("trades@").unwrap_or("");
        info!("Client subscribed to trades for pool: {}", pool_id);
    } else if room.starts_with("klines@") {
        let room_parts: Vec<&str> = room.split("@").collect();
        if room_parts.len() >= 2 {
            let pool_id = room_parts[1];
            let interval = room_parts.get(2).unwrap_or(&"1m");
            info!("Client subscribed to klines for pool: {} with interval: {}", pool_id, interval);
        } else {
            let pool_id = room.strip_prefix("klines@").unwrap_or("");
            info!("Client subscribed to klines for pool: {}", pool_id);
        }
    }
}

async fn handle_unsubscribe_room(room: String, state: &SharedState) {
    // Remove subscriber from the room's subscriber list
    {
        let mut subscribers = state.room_subscribers.write().await;
        if let Some(_room_subscribers) = subscribers.get_mut(&room) {
            // In a real implementation, you'd need to identify which specific sender to remove
            // For now, we'll just log the unsubscription
            info!("Client unsubscribed from room: {}", room);
        }
    }

    // Send confirmation message
    let confirmation = ServerMessage::UnsubscriptionConfirmed {
        room: room.clone(),
    };
    if let Ok(_json) = serde_json::to_string(&confirmation) {
        // Note: In a real implementation, you'd need to send this to the specific client
        info!("Unsubscription confirmed for room: {}", room);
    }
}

// Removed database operations - we only stream data from Redis
