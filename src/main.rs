use std::{sync::{atomic::{self, Ordering}, Arc, Mutex}};
use axum::{body::Body, extract::{ws::WebSocket, State, WebSocketUpgrade}, http::StatusCode, response::Response, routing::any, Json, Router};
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

async fn is_playing_check(State(state): State<Arc<AppState>>,) -> (StatusCode, Json<serde_json::Value>) {
    let is_playing = state.is_playing.load(Ordering::Relaxed);
    let response = serde_json::json!({ "is_playing": is_playing });
    (StatusCode::OK, Json(response))
}

async fn get_last_track(State(state): State<Arc<AppState>>) -> (StatusCode, Json<serde_json::Value>) {
    let last_track = state.last_track_info.lock().unwrap();
    let response = match &*last_track {
        Some(track) => serde_json::json!({
            "track": track,
        }),
        None => serde_json::json!({ "track": null }),
    };
    (StatusCode::OK, Json(response))
}

async fn last_update(State(state): State<Arc<AppState>>,) -> (StatusCode, Json<serde_json::Value>) {
    let last_update = state.last_update.lock().unwrap();
    let response = serde_json::json!({ "last_update": last_update.elapsed().as_secs() });
    (StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_thread_ids(true)
        .with_thread_names(true)
        .init();

    const CLIENT_ID: &str = "1400478980259315843";

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

    utils::discord_rpc_task(state.clone(), CLIENT_ID);

    let app = Router::new()
        .route("/api/ws", any(ws_handler))
        .route("/api/is_playing", any(is_playing_check))
        .route("/api/last_track", any(get_last_track))
        .route("/api/last_update", any(last_update))
        .route("/overlay", any(|| async {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(Body::from(include_str!("../static/overlay.html")))
                .unwrap()
        }))
        .route("/overlay-scroll", any(|| async {
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "text/html")
                .body(Body::from(include_str!("../static/overlay-scroll.html")))
                .unwrap()
        }))
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