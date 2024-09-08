mod common;
mod args;
mod config;
mod actions;
use std::collections::HashMap;
use colored::Colorize;

const DEFAULT_EPISODE_COUNT: usize = 3;

use std::path::Path;
use {
    crate::{
        common::*,
        args::*,
        config::*,
        actions::*,
    },
};


fn select_podcasts(podcasts: Vec<Podcast>, podcast_id_select: &Option<String>) -> Vec<Podcast> {
    podcasts.into_iter()
        .filter(|podcast| match podcast_id_select {
            None => true,
            Some(p_id) => p_id == &podcast.id
        }).collect()
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
            let no_episodes: Vec<Episode> = Vec::new();
            let podcasts: Vec<Podcast> = select_podcasts(settings.podcasts, &args.podcast_id);
            let mut new_state: HashMap<String, Vec<Episode>> = podcasts.into_iter()
                .filter_map(|podcast| {
                    let prev_downloaded_episodes = previous_state.get(&podcast.id).unwrap_or(&no_episodes);
                    match get_episodes(&podcast.url, args.count.unwrap_or(DEFAULT_EPISODE_COUNT)) {
                        Ok(episodes) => {
                            let dir_path = Path::new(&settings.config.media_dir).join(&podcast.id);
                            match download_podcast(episodes, &dir_path, prev_downloaded_episodes) {
                                Ok(downloaded_episodes) => {
                                    println!("{:?} {} episodes downloaded", podcast.id, downloaded_episodes.len());
                                    Some((podcast.id, downloaded_episodes))
                                },
                                Err(e) => {
                                    eprintln!("Could not download the podcast {}: {}", podcast.id, e);
                                    None
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("Could not get the podcast feed: {}", e);
                            None
                        },
                    }
                }).collect();
            // Merge the previous state into the new state to get the updated current state.
            // App state consists of what episodes were downloaded for what podcasts.
            // To avoid storing infinite history, truncate it to latest 100 episodes.
            for (podcast_id, previous_episodes) in previous_state.into_iter() {
                new_state.entry(podcast_id).and_modify(|new_episodes| {
                    new_episodes.extend_from_slice(previous_episodes.as_slice());
                    new_episodes.truncate(100);
                }).or_insert(previous_episodes);
            }
            match store_state(new_state) {
                Ok(()) => println!("All is well."),
                Err(e) => eprintln!("Error while storing the app state {:?}", e)
            }
        },
        Action::REMOTE => {
            let podcasts: Vec<Podcast> = select_podcasts(settings.podcasts, &args.podcast_id);
            for podcast in podcasts.into_iter() {
                match get_episodes(&podcast.url, args.count.unwrap_or(DEFAULT_EPISODE_COUNT)) {
                    Ok(episodes) => {
                        println!("\n{}:", podcast.id.magenta().bold());
                        for episode in episodes.iter() {
                            println!("{}", episode);
                        }
                    },
                    Err(e) => {
                        eprintln!("Could not get the podcast feed: {}", e);
                    },
                }
            }
        },
        Action::LOCAL => todo!(),
        Action::PLAY => {
            let podcasts: Vec<Podcast> = select_podcasts(settings.podcasts, &args.podcast_id);
            for podcast in podcasts.into_iter() {
                match get_episodes(&podcast.url, 1) {
                    Ok(episodes) => {
                        if episodes.is_empty() {
                            println!("Episode not available for {}, download it first.", podcast.id);
                        } else {
                            println!("\n{}:", podcast.id);
                            println!("{:?}", episodes[0]);
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to get episodes: {}", e);
                    }
                }
            }
        }
    }
}
