use std::collections::HashMap;
use crate::common::Episode;
use colored::Colorize;
use crate::config::PodcastSetting;

pub fn show_local(podcasts: Vec<PodcastSetting>, count: usize, previous_state: HashMap<String, Vec<Episode>>) {
    let no_episodes: Vec<Episode> = Vec::new();
    for podcast in podcasts.into_iter() {
        let episodes: Vec<&Episode> = previous_state.get(&podcast.id).unwrap_or(&no_episodes).iter().take(count).collect();
        if episodes.len() == 0 {
            println!("\n{}: no episode available. Download it first.", podcast.id.magenta().bold());
        } else {
            println!("\n{}:", podcast.id.magenta().bold());
            for episode in episodes.into_iter() {
                println!("{}", episode);
            }
        }
    }
}
