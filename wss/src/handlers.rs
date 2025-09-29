use crate::{state::SharedState, websocket::handle_socket};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}
