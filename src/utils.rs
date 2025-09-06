use std::{sync::Arc, thread::sleep, time::Duration};

use clap::Parser;
use tracing::{info, warn};

use crate::models::{AppState, Args, TrackInfo, TrackInfoC};
use std::sync::atomic::Ordering;


#[link(name = "macos-helper")]
extern "C" {
    fn is_music_playing() -> bool;
    fn get_current_track_info() -> TrackInfoC;
    fn free_track_info(info: *mut TrackInfoC);
}

pub fn listen_for_track(state: Arc<AppState>) {
    let state_clone = state.clone();
    info!("Starting track listener thread");
    
    tokio::task::spawn_blocking(move || {
        unsafe {
            let mut track_info = TrackInfoC {
                track_name: std::ptr::null(),
                artist_name: std::ptr::null(),
                progress: 0.0,
                duration: 0.0,
                genre: std::ptr::null(),
                favourited: false,
                played_count: 0,
                album: std::ptr::null(),
            };

            loop {
                if is_music_playing() {
                    free_track_info(&mut track_info);
                    track_info = get_current_track_info();
                    
                    if !track_info.track_name.is_null() {
                        let track_name = std::ffi::CStr::from_ptr(track_info.track_name).to_string_lossy();
                        let artist_name = if !track_info.artist_name.is_null() {
                            std::ffi::CStr::from_ptr(track_info.artist_name).to_string_lossy()
                        } else {
                            "Unknown".into()
                        };
                        let genre = if !track_info.genre.is_null() {
                            std::ffi::CStr::from_ptr(track_info.genre).to_string_lossy()
                        } else {
                            "Unknown".into()
                        };
                        let album = if !track_info.album.is_null() {
                            std::ffi::CStr::from_ptr(track_info.album).to_string_lossy()
                        } else {
                            "Unknown".into()
                        };

                        if let Some(last_info) = &*state_clone.last_track_info.lock().unwrap() {
                            if last_info.track_name == track_name && last_info.artist_name == artist_name {
                                sleep(Duration::from_secs(1));
                                continue;
                            }
                        }

                        let new_track = TrackInfo {
                            track_name: track_name.to_string(),
                            artist_name: artist_name.to_string(),
                            progress: track_info.progress,
                            duration: track_info.duration,
                            genre: genre.to_string(),
                            favourited: track_info.favourited,
                            played_count: track_info.played_count,
                            album: album.to_string(),
                        };

                        {
                            let mut last_info = state_clone.last_track_info.lock().unwrap();
                            *last_info = Some(new_track.clone());
                        }

                        *state_clone.last_update.lock().unwrap() = std::time::Instant::now();
                        state_clone.is_playing.store(true, Ordering::SeqCst);
                        state_clone.scrobble_sent.store(false, Ordering::SeqCst);

                        let _ = state_clone.client_sender.send(new_track);
                    } else {
                        if let Some(last_info) = &*state_clone.last_track_info.lock().unwrap() {
                            if last_info.track_name == "Radio/Mix" && last_info.artist_name == "Unknown" {
                                sleep(Duration::from_secs(1));
                                continue;
                            }
                        }

                        info!("Current track name is null. Most likely radio or mix is playing.");
                        
                        let net_track = TrackInfo {
                            track_name: "Radio/Mix".to_string(),
                            artist_name: "Unknown".to_string(),
                            progress: 0.0,
                            duration: 0.0,
                            genre: "Unknown".to_string(),
                            favourited: false,
                            played_count: 0,
                            album: "Unknown".to_string(),
                        };
                        
                        {
                            let mut last_info = state_clone.last_track_info.lock().unwrap();
                            *last_info = Some(net_track.clone());
                        }
                        
                        *state_clone.last_update.lock().unwrap() = std::time::Instant::now();
                        state_clone.is_playing.store(true, Ordering::SeqCst);
                        let _ = state_clone.client_sender.send(net_track);
                    }
                }

                sleep(Duration::from_secs(1));
            }
        }
    });
}

pub fn parse_args() -> Args {
    let mut args = Args::parse();

    if args.host.to_lowercase() == "localhost" {
        warn!("host is localhost; using 127.0.0.1");
        args.host = "127.0.0.1".to_string();
    }
    args
}