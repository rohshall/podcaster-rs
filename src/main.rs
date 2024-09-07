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

    match args.action {
        Action::DOWNLOAD => {
            let current_state = match get_previous_state() {
                Ok(s) => s,
                Err(_) => HashMap::new()
            };
            let mut new_state: HashMap<String, Vec<String>> = HashMap::new();
            for podcast in config.podcasts.into_iter() {
                match &args.podcast_id {
                    Some(p_id) if p_id != &podcast.id => (),
                    _ => {
                        match get_episodes(&podcast.url, args.count.unwrap_or(3)) {
                            Ok(episodes) => {
                                let dir_path = Path::new(&config.config.media_dir).join(&podcast.id);
                                match download_podcast(episodes, &dir_path, current_state.get(&podcast.id).unwrap_or(&Vec::new())) {
                                    Ok(downloaded_episodes) => {
                                        println!("{:?} {} episodes available", podcast.id, downloaded_episodes.len());
                                        new_state.insert(podcast.id, downloaded_episodes.into_iter().map(|e| e.guid).collect());
                                    },
                                    Err(e) => {
                                        eprintln!("Could not download the podcast {}: {}", podcast.id, e);
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Could not get the podcast feed: {}", e);
                                return;
                            },
                        }
                    },
                }
            }
            // Merge the current state into the new state to get the final state.
            // App state consists of what episodes were downloaded for what podcasts.
            for (podcast_id, current_guids) in current_state.into_iter() {
                new_state.entry(podcast_id).and_modify(|new_guids| new_guids.extend_from_slice(current_guids.as_slice())).or_insert(current_guids);
            }
            match store_state(new_state) {
                Ok(()) => println!("Updated app state stored."),
                Err(e) => println!("Error while storing the app state {:?}", e)
            }
        },
        Action::SHOW => {
            for podcast in config.podcasts.into_iter() {
                match &args.podcast_id {
                    Some(p_id) if p_id != &podcast.id => (),
                    _ => {
                        match get_episodes(&podcast.url, args.count.unwrap_or(3)) {
                            Ok(episodes) => {
                                println!("\n{}:", podcast.id);
                                for episode in episodes.iter() {
                                    println!("{}", episode);
                                }
                            },
                            Err(e) => {
                                eprintln!("Could not get the podcast feed: {}", e);
                                return;
                            },
                        }
                    },
                }
            }

        },
    }
}
