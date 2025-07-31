use std::{thread::sleep, time::Duration};

#[repr(C)]
struct TrackInfo {
    track_name: *const std::os::raw::c_char,
    artist_name: *const std::os::raw::c_char,
    progress: f64,
    duration: f32,
    genre: *const std::os::raw::c_char,
    favourited: bool,
    played_count: i32,
    album: *const std::os::raw::c_char,
}

#[link(name = "macos-helper")]
extern "C" {
    fn is_music_playing() -> bool;
    fn get_current_track_info() -> TrackInfo;
    fn free_track_info(info: *mut TrackInfo);
}

fn main() {
    unsafe {
        let mut track_info = TrackInfo {
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

                    println!("Now playing: {} by {}", track_name, artist_name);
                    println!("Progress: {:.2} seconds / {:.2} seconds", track_info.progress, track_info.duration);
                    println!("Genre: {}", genre);
                    println!("Favourited: {}", track_info.favourited);
                    println!("Played Count: {}", track_info.played_count);
                    println!("Album: {}", album);
                }
            } else {
                println!("Music is not currently playing.");
            }
            // sleep(Duration::from_secs(1));
        }
    }
}