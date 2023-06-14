mod args;
mod config;
mod downloader;

use std::path::Path;
use {
    crate::{
        args::*,
        config::*,
        downloader::*,
    },
};

fn main() {
    let args: Args = argh::from_env();
    if args.version {
        println!("podcaster {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let podcast_id = match args.podcast_id {
        Some(pid) => pid,
        None => {
            eprintln!("podcastId from the config file required");
            return;
        }
    };
    let config: Config = match get_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not parse config file: {}", e);
            return;
        }
    };

    let podcast_url = match config.podcasts.get(&podcast_id) {
        Some(url) => url,
        None => {
            eprintln!("Invalid podcast-id");
            return;
        }
    };
    let dir_path = Path::new(&config.media_dir).join(&podcast_id);
    match download_podcast(podcast_url, &dir_path) {
        Ok(()) => {
            println!("Download complete");
        },
        Err(e) => {
            eprintln!("Could not download the podcast episode: {}", e);
            return;
        }
    };
}

