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

#[link(name = "helper")]
extern "C" {
    fn is_music_playing() -> bool;
    fn get_current_track_info() -> TrackInfo;
}

fn main() {
    unsafe {
        loop {
            if is_music_playing() {
                let track_info = get_current_track_info();
                if !track_info.track_name.is_null() {
                    let track_name = std::ffi::CStr::from_ptr(track_info.track_name).to_string_lossy();
                    let artist_name = std::ffi::CStr::from_ptr(track_info.artist_name).to_string_lossy();
                    let genre = std::ffi::CStr::from_ptr(track_info.genre).to_string_lossy();
                    let album = std::ffi::CStr::from_ptr(track_info.album).to_string_lossy();

                    println!("Now playing: {} by {}", track_name, artist_name);
                    println!("Progress: {:.2} seconds / {:.2} seconds", track_info.progress, track_info.duration);
                    println!("Genre: {}", genre);
                    println!("Favourited: {}", track_info.favourited);
                    println!("Played Count: {}", track_info.played_count);
                    println!("Album: {}", album);

                    libc::free(track_info.track_name as *mut libc::c_void);
                    libc::free(track_info.artist_name as *mut libc::c_void);
                    libc::free(track_info.genre as *mut libc::c_void);
                    libc::free(track_info.album as *mut libc::c_void);
                }
            } else {
                println!("Music is not currently playing.");
            }
        }
    }
}