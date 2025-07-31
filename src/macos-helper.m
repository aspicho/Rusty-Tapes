#import <Foundation/Foundation.h>
#import <ScriptingBridge/ScriptingBridge.h>
#import <string.h>

#define PLAYING 1800426320

typedef struct {
    const char* track_name;
    const char* artist_name;
    double progress;
    float duration;
    const char* genre;
    bool favourited;
    int played_count;
    const char* album;
} TrackInfo;

static Class SBApplicationClass = nil;
static id musicApp = nil;
static bool debug = false;

void set_debug_mode(bool enable) {
    debug = enable;
}

void initialize_music_app(void) {
    if (!SBApplicationClass) {
        SBApplicationClass = NSClassFromString(@"SBApplication");
        if (!SBApplicationClass) {
            if (debug) NSLog(@"SBApplication class not found");
            return;
        }
    }

    if (!musicApp) {
        musicApp = [SBApplicationClass applicationWithBundleIdentifier:@"com.apple.Music"];
        if (!musicApp) {
            if (debug) NSLog(@"Music app not found");
        }
    }
}

bool is_music_playing(void) {
    initialize_music_app();
    if (!musicApp) {
        return false;
    }

    int playerState = [[musicApp valueForKey:@"playerState"] intValue];
    if (debug) NSLog(@"Player state: %d", playerState);
    return playerState == PLAYING;
}

TrackInfo get_current_track_info(void) {
    initialize_music_app();
    TrackInfo info = {0};

    if (!musicApp) {
        return info;
    }

    id currentTrack = [musicApp valueForKey:@"currentTrack"];
    if (currentTrack) {
        NSString *trackName = [currentTrack valueForKey:@"name"];
        NSString *artistName = [currentTrack valueForKey:@"artist"];
        NSString *genre = [currentTrack valueForKey:@"genre"];
        NSString *album = [currentTrack valueForKey:@"album"];

        info.track_name = trackName ? strdup([trackName UTF8String]) : NULL;
        info.artist_name = artistName ? strdup([artistName UTF8String]) : NULL;
        info.genre = genre ? strdup([genre UTF8String]) : NULL;
        info.album = album ? strdup([album UTF8String]) : NULL;

        info.progress = [[musicApp valueForKey:@"playerPosition"] doubleValue];
        info.duration = [[currentTrack valueForKey:@"duration"] floatValue];
        info.favourited = [[currentTrack valueForKey:@"favorited"] boolValue];
        info.played_count = [[currentTrack valueForKey:@"playedCount"] intValue];

        if (debug) {
            NSLog(@"Track Info: Name=%s, Artist=%s, Progress=%.2f, Duration=%.2f, Genre=%s, Favourited=%d, Played Count=%d, Album=%s",
                  info.track_name ? info.track_name : "null",
                  info.artist_name ? info.artist_name : "null",
                  info.progress,
                  info.duration,
                  info.genre ? info.genre : "null",
                  info.favourited,
                  info.played_count,
                  info.album ? info.album : "null");
        }
    }
    return info;
}

void free_track_info(TrackInfo* info) {
    if (debug) NSLog(@"Entering free_track_info");
    if (!info) {
        if (debug) NSLog(@"TrackInfo pointer is null");
        return;
    }

    if (info->track_name) {
        if (debug) NSLog(@"Freeing track_name: %s", info->track_name);
        free((void*)info->track_name);
        info->track_name = NULL;
    } else {
        if (debug) NSLog(@"track_name is already null");
    }

    if (info->artist_name) {
        if (debug) NSLog(@"Freeing artist_name: %s", info->artist_name);
        free((void*)info->artist_name);
        info->artist_name = NULL;
    } else {
        if (debug) NSLog(@"artist_name is already null");
    }

    if (info->genre) {
        if (debug) NSLog(@"Freeing genre: %s", info->genre);
        free((void*)info->genre);
        info->genre = NULL;
    } else {
        if (debug) NSLog(@"genre is already null");
    }

    if (info->album) {
        if (debug) NSLog(@"Freeing album: %s", info->album);
        free((void*)info->album);
        info->album = NULL;
    } else {
        if (debug) NSLog(@"album is already null");
    }

    if (debug) NSLog(@"Exiting free_track_info");
}