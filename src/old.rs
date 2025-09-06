#[macro_use]
extern crate objc;

use cocoa::base::{nil, id};
use cocoa::foundation::{NSAutoreleasePool, NSString};
use std::ffi::CStr;
use std::thread::sleep;
use std::time::Duration;

// do not build on windows 
#[cfg(not(target_os = "windows"))]
fn main() {
    unsafe {
        // Create an initial autorelease pool (used for one-time setup)
        let _pool = NSAutoreleasePool::new(nil);

        // Load the ScriptingBridge framework once.
        let bundle_path = NSString::alloc(nil).init_str("/System/Library/Frameworks/ScriptingBridge.framework");
        let ns_bundle_class = objc::runtime::Class::get("NSBundle").expect("Failed to get NSBundle class");
        let bundle: id = msg_send![ns_bundle_class, bundleWithPath: bundle_path];
        let _: () = msg_send![bundle, load];

        // Get the Music app via SBApplication once.
        let sb_app_class = objc::runtime::Class::get("SBApplication")
            .expect("Failed to get SBApplication class");
        let bundle_identifier = NSString::alloc(nil).init_str("com.apple.Music");
        let music_app: id = msg_send![sb_app_class, applicationWithBundleIdentifier: bundle_identifier];

        // Start an infinite loop.
        loop {
            // Create an autorelease pool for this iteration.
            let inner_pool = NSAutoreleasePool::new(nil);

            // Check if Music is playing.
            let player_state: i32 = msg_send![music_app, playerState];
            const PLAYING: i32 = 1800426320; // The constant for "playing".

            if player_state == PLAYING {
                // Get the current track.
                let current_track: id = msg_send![music_app, currentTrack];

                // Retrieve properties.
                let track_name: id = msg_send![current_track, name];
                let artist_name: id = msg_send![current_track, artist];
                let progress: f64 = msg_send![music_app, playerPosition];
                let duration: f64 = msg_send![current_track, duration];
                let genre: id = msg_send![current_track, genre];
                let favorited: bool = msg_send![current_track, favorited];
                let played_count: i32 = msg_send![current_track, playedCount];
                let album: id = msg_send![current_track, album];

                // Convert NSString values to Rust strings.
                let track_name_str = nsstring_to_rust_string(track_name);
                let artist_name_str = nsstring_to_rust_string(artist_name);
                let genre_str = nsstring_to_rust_string(genre);
                let album_str = nsstring_to_rust_string(album);

                println!("Now playing: {} by {}", track_name_str, artist_name_str);
                println!("Duration: {} seconds | Progress: {} seconds", duration, progress);
                println!("Genre: {}", genre_str);
                println!("Favorited: {}", favorited);
                println!("Played Count: {}", played_count);
                println!("Album: {}", album_str);
            } else {
                println!("Music is not currently playing.");
            }

            // Drain the pool for this iteration.
            inner_pool.drain();

            // Sleep for a second before the next check.
            sleep(Duration::from_secs(1));
        }
    }
}

/// Converts an Objective-C NSString to a Rust String.
/// 
#[allow(non_snake_case)]
#[cfg(not(target_os = "windows"))]
fn nsstring_to_rust_string(ns_string: id) -> String {
    unsafe {
        if ns_string == nil {
            return String::new();
        }
        // Get a pointer to a C string from the NSString.
        let c_str: *const std::os::raw::c_char = msg_send![ns_string, UTF8String];
        if c_str.is_null() {
            String::new()
        } else {
            CStr::from_ptr(c_str).to_string_lossy().into_owned()
        }
    }
}