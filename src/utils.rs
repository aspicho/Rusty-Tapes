use std::{os::macos::raw::stat, sync::Arc, thread::sleep, time::Duration};

use clap::Parser;
use serde_json::json;
use tracing::{info, warn};

use crate::models::{AppState, Args, TrackInfo, TrackInfoC};
use std::sync::atomic::Ordering;
use yet_another_discord_rpc::DiscordRpc;

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

            let mut last_track_identifier: Option<String> = None;
            let mut was_playing = false;

            loop {
                let is_currently_playing = is_music_playing();
                let play_state_changed = was_playing != is_currently_playing;
                
                if is_currently_playing {
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

                        let track_identifier = format!("{}|{}", track_name, artist_name);
                        let track_changed = last_track_identifier.as_ref() != Some(&track_identifier);

                        if track_changed || play_state_changed {
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
                            
                            if track_changed {
                                state_clone.scrobble_sent.store(false, Ordering::SeqCst);
                                last_track_identifier = Some(track_identifier);
                            }

                            let _ = state_clone.client_sender.send(new_track);
                        }
                    } else {
                        let track_identifier = "Radio/Mix|Unknown".to_string();
                        let track_changed = last_track_identifier.as_ref() != Some(&track_identifier);

                        if track_changed || play_state_changed {
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
                            last_track_identifier = Some(track_identifier);
                        }
                    }
                } else if play_state_changed {
                    state_clone.is_playing.store(false, Ordering::SeqCst);
                    if let Some(last_info) = &*state_clone.last_track_info.lock().unwrap() {
                        let pause_track = TrackInfo {
                            progress: 0.0,
                            duration: -1.0,
                            ..last_info.clone()
                        };
                        let _ = state_clone.client_sender.send(pause_track);
                    }
                }

                was_playing = is_currently_playing;
                sleep(Duration::from_secs(1));
            }
        }
    });
}

pub fn discord_rpc_task(state: Arc<AppState>, client_id: &str) {
    info!("Starting Discord RPC task");

    let state_clone = state.clone();
    let client_id = client_id.to_string();
    tokio::spawn(async move {
        let mut rpc = DiscordRpc::new(&client_id).await.expect("Failed to create Discord RPC client");
        rpc.start_activity(None).await.expect("Failed to start activity");

        let mut reciever = state_clone.client_sender.subscribe();
        while let Ok(track) = reciever.recv().await {

            let resp = reqwest::get(
                format!("https://itunes.apple.com/search?term={}&country=us&limit=25&media=music&entity=musicTrack&attribute=songTerm",
                    urlencoding::encode(&track.track_name)
                )
            ).await;

            let image_url: Option<String> = match resp {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        json.get("results")
                            .and_then(|r| r.as_array())
                            .and_then(|results| {
                                results.iter().find_map(|result| {
                                    let artist = result.get("artistName")?.as_str()?;
                                    let track_name = result.get("trackName")?.as_str()?;
                                    if artist.eq_ignore_ascii_case(&track.artist_name) 
                                        && track_name.eq_ignore_ascii_case(&track.track_name) {
                                        let artwork = result.get("artworkUrl100")?.as_str()?;
                                        Some(artwork.replace("100x100bb.jpg", "512x512bb.jpg"))
                                    } else {
                                        None
                                    }
                                })
                            })
                    } else {
                        None
                    }
                }
                Err(_) => None,
            };

            let mut activity = json!({
                "type": 2,
                "details": format!("Listening to {}", track.track_name),
                "state": format!("by {}", track.artist_name),
                "assets": {
                    "large_image": image_url.unwrap_or_else(|| "image_logo".to_string()),
                    "large_text": format!("Apple Music{}{}", 
                        if track.genre != "Unknown" { format!(" - {}", track.genre) } else { "".to_string() }, 
                        if track.played_count > 0 { format!(" (Played {} times)", track.played_count) } else { "".to_string() }),
                    "small_image": if track.favourited { "favourite" } else { "unfavourite" },
                    "small_text": if track.favourited { "Favourited" } else { "Not Favourited" },
                },
            });

            if track.duration > 0.0 {
                let start_timestamp = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as f64
                    - track.progress) as i64;
                let end_timestamp = start_timestamp + track.duration as i64;

                activity["timestamps"] = json!({
                    "start": start_timestamp,
                    "end": end_timestamp,
                });
            }

            rpc.set_activity(activity).await.expect("Failed to set activity");
            info!("Updated Discord RPC activity for track: {} by {}", track.track_name, track.artist_name);
        }
        rpc.stop_activity().await.expect("Failed to stop activity");
        info!("Stopped Discord RPC activity");
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