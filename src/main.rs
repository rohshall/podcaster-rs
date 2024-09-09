mod common;
mod args;
mod config;
mod actions;
use std::collections::HashMap;
use colored::Colorize;
use std::path::Path;
use std::process::{Command, Stdio};
use {
    crate::{
        common::*,
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
            let no_episodes: Vec<Episode> = Vec::new();
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let count = args.count.unwrap_or(1);
            let media_dir = Path::new(&settings.config.media_dir);
            let new_state: HashMap<String, Vec<Episode>> = podcasts.into_iter()
                .filter_map(|podcast| {
                    let prev_downloaded_episodes = previous_state.get(&podcast.id).unwrap_or(&no_episodes);
                    let dir_path = media_dir.join(&podcast.id);
                    match get_episodes(&podcast.url, count) {
                        Ok(episodes) => {
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
            let mut updated_state: HashMap<String, Vec<Episode>> = HashMap::new();
            for (podcast_id, new_episodes) in new_state.into_iter() {
                let mut new_episodes = new_episodes.clone();
                let previous_episodes = previous_state.get(&podcast_id).unwrap_or(&no_episodes);
                new_episodes.extend_from_slice(previous_episodes.as_slice());
                updated_state.insert(podcast_id, new_episodes);
            }
            for (podcast_id, old_episodes) in previous_state.into_iter() {
                if updated_state.get(&podcast_id).is_none() {
                    updated_state.insert(podcast_id, old_episodes);
                }
            }
            match store_state(updated_state) {
                Ok(()) => println!("All is well."),
                Err(e) => eprintln!("Error while storing the app state {:?}", e)
            }
        },
        Action::REMOTE => {
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let count = args.count.unwrap_or(5);
            for podcast in podcasts.into_iter() {
                match get_episodes(&podcast.url, count) {
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
            let no_episodes: Vec<Episode> = Vec::new();
            let podcasts = select_podcasts(settings.podcasts, args.podcast_id);
            let media_dir = Path::new(&settings.config.media_dir);
            let count = args.count.unwrap_or(1);
            let player = settings.config.player;
            for podcast in podcasts.iter() {
                let episodes: Vec<&Episode> = previous_state.get(&podcast.id).unwrap_or(&no_episodes).iter().take(count).collect();
                let dir_path = media_dir.join(&podcast.id);
                if episodes.len() == 0 {
                    println!("\n{}: no episode available. Download it first.", podcast.id.magenta().bold());
                } else {
                    println!("\n{}:", podcast.id.magenta().bold());
                    for episode in episodes.into_iter() {
                        match get_episode_download(episode, &dir_path) {
                            Some(path) => {
                                let child = Command::new(&player)
                                    .arg(path.display().to_string())
                                    .stdout(Stdio::piped())
                                    .spawn()
                                    .expect("failed to execute the player");
                                child
                                    .wait_with_output()
                                    .expect("failed to wait on child");
                            },
                            None => eprintln!("Could not get the file for the episode at URL {}", episode.url)
                        }
                    }
                }
            }
        }
    }
}
