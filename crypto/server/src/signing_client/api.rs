use std::str;

use axum::{
    extract::{
        ws::{WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{signing_client::protocol_transport::handle_socket, AppState};

pub const SUBSCRIBE_TIMEOUT_SECONDS: u64 = 10;

/// Handle an incoming websocket connection
pub async fn ws_handler(
    State(app_state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket_result(socket, app_state))
}

async fn handle_socket_result(socket: WebSocket, app_state: AppState) {
    if let Err(err) = handle_socket(socket, app_state).await {
        tracing::warn!("Websocket connection closed unexpectedly {:?}", err);
        // TODO here we should inform the chain that signing failed
    };
}

use serde::{Deserialize, Serialize};

// TODO: JA remove all below temporary
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Clone)]
pub struct Message {
    pub message: String,
}

/// Returns the signature of the requested sighash
///
/// This will be removed when after client participates in signing
pub async fn get_signature(
    State(app_state): State<AppState>,
    Json(msg): Json<Message>,
) -> (StatusCode, String) {
    let sig = match app_state.signature_state.get(&hex::decode(msg.message).unwrap()) {
        Some(sig) => sig,
        None => return (StatusCode::NO_CONTENT, "".to_string()),
    };
    (StatusCode::ACCEPTED, base64::encode(sig.to_rsv_bytes()))
}

/// Drains all signatures from the state
/// Client calls this after they receive the signature at `/signature`
///
/// This will be removed when after client participates in signing
pub async fn drain(State(app_state): State<AppState>) -> Result<StatusCode, ()> {
    app_state.signature_state.drain();
    Ok(StatusCode::OK)
}
