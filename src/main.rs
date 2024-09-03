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
    let config: Settings = match get_config() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not parse config file: {}", e);
            return;
        }
    };

    for podcast in config.podcasts.into_iter() {
        match &args.podcast_id {
            Some(p_id) if p_id != &podcast.id => (),
            _ => {
                let dir_path = Path::new(&config.config.media_dir).join(&podcast.id);
                match download_podcast(&podcast.url, &dir_path, args.count.unwrap_or(3)) {
                    Ok(()) => {
                        println!("Download complete");
                    },
                    Err(e) => {
                        eprintln!("Could not download the podcast episode: {}", e);
                        return;
                    }
                };
            }

        };
    }

}
