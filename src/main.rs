mod common;
mod args;
mod config;
mod actions;
use std::collections::HashMap;
use std::path::Path;
use {
    crate::{
        args::*,
        config::*,
        actions::*,
    },
};

fn select_podcasts(podcasts: Vec<Podcast>, podcast_id: Option<String>) -> Vec<Podcast> {
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
            let media_dir = &settings.config.media_dir;
            let new_state = download_podcasts(podcasts, &media_dir, count, &previous_state);
            let updated_state = compute_updated_state(new_state, previous_state);
            match store_state(updated_state) {
                Ok(()) => println!("All is well."),
                Err(e) => eprintln!("Error while storing the app state {:?}", e)
            }
        },
        Action::REMOTE => {
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let count = args.count.unwrap_or(5);
            show_remote(podcasts, count);
        },
        Action::LOCAL => {
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let count = args.count.unwrap_or(5);
            show_local(podcasts, count, previous_state);
        },
        Action::PLAY => {
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let media_dir = Path::new(&settings.config.media_dir);
            let count = args.count.unwrap_or(1);
            let player = settings.config.player;
            let speed = settings.config.speed;
            play_podcasts(podcasts, count, media_dir, player, speed, previous_state);
        }
    }
}
