use std::time::Duration;
use std::thread;
use std::thread::ScopedJoinHandle;
use std::vec::Vec;
use std::path::PathBuf;
use std::collections::HashMap;
use ureq;
use std::sync::{Arc, Mutex};
use linya::Progress;
use std::process::{Command, Stdio};
use crate::settings::Settings;
use crate::podcast::Podcast;

#[derive(Debug)]
pub struct Podcaster {
    // Model podcasts as a HashMap to quickly find out the podcast based on the podcast ID.
    podcasts: HashMap<String, Podcast>,
    agent: ureq::Agent,
    // Playback settings
    player: String,
}

impl Podcaster {
    pub fn new() -> Podcaster {
        let settings = Settings::new().expect("Failed to parse the config file");
        // Store the config params.
        let media_dir = settings.media_dir;
        let player = settings.player;
        // Create an HTTP client agent to be used for downloaing podcasts.
        let agent = ureq::AgentBuilder::new()
            .redirects(8)
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();
        // Store podcasts as a HashMap with the podcast ID as the key so that we can find the
        // podcast using the podcast ID.
        let podcasts: HashMap<String, Podcast> = settings.podcasts.into_iter().map(|(podcast_id, podcast_url)| {
            (podcast_id.clone(), Podcast::new(podcast_id, podcast_url, &media_dir))
        }).collect();
        Self { podcasts, agent, player }
    }

    // Utility function to select podcasts based on the podcast ID. If no podcast ID is specified,
    // that means we are looking at all the podcasts from the settings.
    fn select_podcasts(&self, podcast_id: Option<String>) -> Vec<&Podcast> {
        match podcast_id {
            None => self.podcasts.values().collect(),
            Some(p_id) => self.podcasts.get(&p_id).into_iter().collect()
        }
    }

    // Handle download action.
    pub fn download(&self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(1);
        let progress = Arc::new(Mutex::new(Progress::new()));
        // Download the podcasts concurrently.
        thread::scope(|s| {
            let handles: Vec<ScopedJoinHandle<()>> = podcasts.into_iter().map(|podcast| {
                let progress = Arc::clone(&progress);
                s.spawn(move || {
                    podcast.download(&self.agent, progress, count);
                })}).collect();
            for handle in handles {
                handle.join().unwrap();
            }
        });
    }

    // Handle catchup action.
    pub fn catchup(&self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(5);
        for podcast in podcasts.into_iter() {
            podcast.catchup(&self.agent, count);
        }
    }

    // Handle list action.
    pub fn list(&self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(5);
        for podcast in podcasts.into_iter() {
            podcast.list(&self.agent, count);
        }
    }

    // Handle play action.
    pub fn play(&self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(1);
        let player = &self.player;
        // Create a playlist from latest downloaded episodes of the selected podcast(s).
        let playlist: Vec<PathBuf> = podcasts.into_iter().flat_map(|podcast| {
            podcast.files(count).into_iter()
        }).collect();
        if playlist.is_empty() {
            eprintln!("No episodes available to play; download them first.");
        } else {
            println!("Playing episodes: {:?}", playlist);
            let child = Command::new(&player)
                .args(playlist)
                .stdout(Stdio::piped())
                .spawn()
                .expect("failed to execute the player");
            child
                .wait_with_output()
                .expect("failed to wait on child");
        }
    }

}
