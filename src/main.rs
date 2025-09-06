use std::{sync::{atomic::{self, Ordering}, Arc, Mutex}, thread::sleep, time::Duration};
use axum::{extract::{ws::WebSocket, State, WebSocketUpgrade}, routing::any, Router};
use models::TrackInfo;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};
use futures_util::{sink::SinkExt, stream::{StreamExt, SplitSink, SplitStream}};


mod models;
mod utils;

use crate::models::AppState;

async fn socket_handler(mut socket: WebSocket, state: Arc<AppState>) {
    let connection_count = state.active_connections.fetch_add(1, Ordering::Relaxed);
    info!("New client connection. Total: {}", connection_count + 1);

    let (sender, receiver) = socket.split();
    let mut message_receiver = state.client_sender.subscribe();

    let state_clone = state.clone();
    let reader_handle = tokio::spawn(async move {
        reader_client_task(receiver, state_clone).await;
    });

    let writer_handle = tokio::spawn(async move {
        writer_client_task(sender, message_receiver).await;
    });

    tokio::select! {
        _ = reader_handle => {
            info!("Client reader task completed first");
        }
        _ = writer_handle => {
            info!("Client writer task completed first");
        }
    }

    let final_count = state.active_connections.fetch_sub(1, Ordering::Relaxed);
    info!("Client connection closed. Total: {}", final_count - 1);
}

async fn reader_client_task(mut receiver: SplitStream<WebSocket>, state: Arc<AppState>) {
    while let Some(msg) = receiver.next().await {
        match msg {
            Ok(msg) => {
                info!("Received client message: {:?}", msg);
            }
            Err(e) => {
                warn!("Error receiving client message: {:?}", e);
                break;
            }
        }
    }
}

async fn writer_client_task(mut sender: SplitSink<WebSocket, axum::extract::ws::Message>, mut message_receiver: broadcast::Receiver<models::TrackInfo>) {
    while let Ok(chat_message) = message_receiver.recv().await {
        let msg_text = serde_json::to_string(&chat_message).unwrap_or_else(|_| "{}".to_string());
        if sender.send(axum::extract::ws::Message::Text(msg_text.into())).await.is_err() {
            warn!("Error sending client message");
            break;
        }
    }
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> axum::response::Response {
    ws.on_upgrade(|socket| socket_handler(socket, state))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    let state = Arc::new(AppState {
        client_sender: {
            let (tx, _rx) = tokio::sync::broadcast::channel(100);
            tx
        },
        active_connections: atomic::AtomicUsize::new(0),
        last_track_info: Mutex::new(None),
        last_update: Mutex::new(std::time::Instant::now()),
        is_playing: atomic::AtomicBool::new(false),
        scrobble_sent: atomic::AtomicBool::new(false),

    });

    utils::listen_for_track(state.clone());

    let app = Router::new()
        .route("/api/ws", any(ws_handler))
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_headers(Any)
                .allow_origin(Any),
        )
        .with_state(state);

    let args = utils::parse_args();

    info!("Server listening on http://{}:{}", args.host, args.port);
    let listener = tokio::net::TcpListener::bind((args.host, args.port)).await
        .expect("Failed to bind TCP listener");

    axum::serve(listener, app).await.unwrap();
}