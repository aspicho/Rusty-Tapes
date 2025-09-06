use std::sync::{atomic, Mutex};

use clap::Parser;

#[repr(C)]
pub struct TrackInfoC {
    pub track_name: *const std::os::raw::c_char,
    pub artist_name: *const std::os::raw::c_char,
    pub progress: f64,
    pub duration: f32,
    pub genre: *const std::os::raw::c_char,
    pub favourited: bool,
    pub played_count: i32,
    pub album: *const std::os::raw::c_char,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TrackInfo {
    pub track_name: String,
    pub artist_name: String,
    pub progress: f64,
    pub duration: f32,
    pub genre: String,
    pub favourited: bool,
    pub played_count: i32,
    pub album: String,
}

pub struct AppState {
    pub client_sender: tokio::sync::broadcast::Sender<TrackInfo>,
    pub active_connections: atomic::AtomicUsize,
    pub last_track_info: Mutex<Option<TrackInfo>>,
    pub last_update: Mutex<std::time::Instant>,
    pub is_playing: atomic::AtomicBool,
    pub scrobble_sent: atomic::AtomicBool,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The address to bind the server to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    pub host: String,
    
    /// The port to bind the server to
    #[arg(short, long, default_value = "7271")]
    pub port: u16,
}