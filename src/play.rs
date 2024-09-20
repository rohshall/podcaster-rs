use url::Url;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use crate::common::Episode;
use colored::Colorize;
use std::process::{Command, Stdio};
use crate::config::PodcastSetting;


fn get_episode_download(episode: &Episode, dir_path: &PathBuf) -> Option<PathBuf> {
    Url::parse(&episode.url.as_str()).ok().and_then(|url| -> Option<PathBuf> {
        let file_name = Path::new(url.path()).file_name().unwrap();
        let path = dir_path.join(file_name);
        match path.try_exists() {
            Ok(exists) => exists.then_some(path),
            Err(_) => None
        }
    })
}

pub fn play_podcasts(podcasts: Vec<PodcastSetting>, count: usize, media_dir: &Path, player: String, speed: f64, previous_state: HashMap<String, Vec<Episode>>) {
    let no_episodes: Vec<Episode> = Vec::new();
    for podcast in podcasts.into_iter() {
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
                            .args([format!("--speed={:.2}", speed), path.display().to_string()])
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
