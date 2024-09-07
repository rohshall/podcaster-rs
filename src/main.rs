mod args;
mod config;
mod actions;
use std::collections::HashMap;

const DEFAULT_EPISODE_COUNT: usize = 3;

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
            let previous_state = match get_state() {
                Ok(s) => s,
                Err(_) => HashMap::new()
            };
            let mut new_state: HashMap<String, Vec<String>> = HashMap::new();

            for podcast in config.podcasts.into_iter() {
                match &args.podcast_id {
                    Some(p_id) if p_id != &podcast.id => (),
                    _ => {
                        match get_episodes(&podcast.url, args.count.unwrap_or(DEFAULT_EPISODE_COUNT)) {
                            Ok(episodes) => {
                                let dir_path = Path::new(&config.config.media_dir).join(&podcast.id);
                                match download_podcast(episodes, &dir_path, previous_state.get(&podcast.id).unwrap_or(&Vec::new())) {
                                    Ok(downloaded_episodes) => {
                                        println!("{:?} {} episodes downloaded", podcast.id, downloaded_episodes.len());
                                        new_state.insert(podcast.id, downloaded_episodes.into_iter().map(|e| e.guid).collect());
                                    },
                                    Err(e) => {
                                        eprintln!("Could not download the podcast {}: {}", podcast.id, e);
                                    }
                                }
                            },
                            Err(e) => {
                                eprintln!("Could not get the podcast feed: {}", e);
                            },
                        }
                    },
                }
            }
            // Merge the previous state into the new state to get the updated current state.
            // App state consists of what episodes were downloaded for what podcasts.
            // To avoid storing infinite history, truncate it to latest 100 episodes.
            for (podcast_id, previous_guids) in previous_state.into_iter() {
                new_state.entry(podcast_id).and_modify(|new_guids| {
                    new_guids.extend_from_slice(previous_guids.as_slice());
                    new_guids.truncate(100);
                }).or_insert(previous_guids);
            }
            match store_state(new_state) {
                Ok(()) => println!("All is well."),
                Err(e) => println!("Error while storing the app state {:?}", e)
            }
        },
        Action::SHOW => {
            for podcast in config.podcasts.into_iter() {
                match &args.podcast_id {
                    Some(p_id) if p_id != &podcast.id => (),
                    _ => {
                        match get_episodes(&podcast.url, args.count.unwrap_or(DEFAULT_EPISODE_COUNT)) {
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
