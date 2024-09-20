mod common;
mod args;
mod config;
mod download;
mod show_local;
mod show_remote;
mod play;
use std::collections::HashMap;
use std::time::Duration;
use std::path::Path;
use ureq;
use {
    crate::{
        args::*,
        config::*
    },
};


fn select_podcasts(podcasts: Vec<PodcastSetting>, podcast_id: Option<String>) -> Vec<PodcastSetting> {
    match podcast_id {
        None => podcasts,
        Some(p_id) => podcasts.into_iter().find(|podcast| p_id == podcast.id).into_iter().collect()
    }
}

fn main() {
    let args: Args = argh::from_env();
    if args.version {
        println!("podcaster {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let settings: Settings = match get_settings() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not parse config file: {}", e);
            return;
        }
    };
    let previous_state = match get_state() {
        Ok(s) => s,
        Err(_) => HashMap::new()
    };

    match args.action {
        Action::DOWNLOAD => {
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let count = args.count.unwrap_or(1);
            let media_dir = &settings.media_dir;
            let agent = ureq::AgentBuilder::new()
                .redirects(8)
                .timeout_read(Duration::from_secs(5))
                .timeout_write(Duration::from_secs(5))
                .build();
            let updated_state = download::download_podcasts(&agent, podcasts, &media_dir, count, previous_state);
            match store_state(updated_state) {
                Ok(()) => println!("All is well."),
                Err(e) => eprintln!("Error while storing the app state {:?}", e)
            }
        },
        Action::REMOTE => {
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let count = args.count.unwrap_or(5);
            show_remote::show_remote(podcasts, count);
        },
        Action::LOCAL => {
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let count = args.count.unwrap_or(5);
            show_local::show_local(podcasts, count, previous_state);
        },
        Action::PLAY => {
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let media_dir = Path::new(&settings.media_dir);
            let count = args.count.unwrap_or(1);
            let player = settings.player;
            let speed = settings.playback_speed;
            play::play_podcasts(podcasts, count, media_dir, player, speed, previous_state);
        }
    }
}
