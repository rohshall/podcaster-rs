mod args;
mod config;
mod actions;

use std::path::Path;
use {
    crate::{
        args::*,
        config::*,
        actions::*,
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
                match get_episodes(&podcast.url, args.count.unwrap_or(3)) {
                    Ok(episodes) => {
                        match args.action.as_str() {
                            "download" => {
                                let dir_path = Path::new(&config.config.media_dir).join(&podcast.id);
                                match download_podcast(&episodes, &dir_path) {
                                    Ok(()) => {
                                        println!("{:?} download complete", podcast.id);
                                    },
                                    Err(e) => {
                                        eprintln!("Could not download the podcast episode: {}", e);
                                        return;
                                    }
                                }
                            },
                            "show" => {
                                println!("\n{}:", podcast.id);
                                for episode in episodes.iter() {
                                    println!("{}", episode);
                                }
                            },
                            _ => {
                                println!("Unknown action!");
                                return;
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Could not get the podcast feed: {}", e);
                        return;
                    },
                }
            }

        };
    }

}
