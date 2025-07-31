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

bool is_music_playing(void) {
    Class SBApplicationClass = NSClassFromString(@"SBApplication");
    if (!SBApplicationClass) {
        return false;
    }

    id musicApp = [SBApplicationClass applicationWithBundleIdentifier:@"com.apple.Music"];
    if (!musicApp) {
        return false;
    }

    int playerState = [[musicApp valueForKey:@"playerState"] intValue];
    return playerState == PLAYING;
}

TrackInfo get_current_track_info(void) {
    TrackInfo info = {0};

    Class SBApplicationClass = NSClassFromString(@"SBApplication");
    if (!SBApplicationClass) {
        return info;
    }

    id musicApp = [SBApplicationClass applicationWithBundleIdentifier:@"com.apple.Music"];
    if (!musicApp) {
        return info;
    }

    id currentTrack = [musicApp valueForKey:@"currentTrack"];
    if (currentTrack) {
        info.track_name = strdup([[currentTrack valueForKey:@"name"] UTF8String]);
        info.artist_name = strdup([[currentTrack valueForKey:@"artist"] UTF8String]);
        info.progress = [[musicApp valueForKey:@"playerPosition"] doubleValue];
        info.duration = [[currentTrack valueForKey:@"duration"] floatValue];
        info.genre = strdup([[currentTrack valueForKey:@"genre"] UTF8String]);
        info.favourited = [[currentTrack valueForKey:@"favorited"] boolValue];
        info.played_count = [[currentTrack valueForKey:@"playedCount"] intValue];
        info.album = strdup([[currentTrack valueForKey:@"album"] UTF8String]);
    }

    return info;
}