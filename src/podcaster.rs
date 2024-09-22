use std::time::Duration;
use std::thread;
use std::vec::Vec;
use std::collections::HashMap;
use ureq;
use std::sync::{Arc, Mutex};
use linya::Progress;
use std::process::{Command, Stdio};
use crate::settings::Settings;
use crate::podcast::Podcast;
use crate::state::{get_state, store_state};

#[derive(Debug)]
pub struct Podcaster {
    // Model podcasts as a HashMap to quickly find out the podcast based on the podcast ID.
    podcasts: HashMap<String, Podcast>,
    player: String,
    playback_speed: f64,
}

impl Podcaster {
    pub fn new() -> Podcaster {
        let settings = Settings::new().expect("Failed to parse the config file: ~/.podcasts.toml");
        // Store the config params.
        let media_dir = settings.media_dir;
        let player = settings.player;
        let playback_speed = settings.playback_speed;
        // Create an HTTP client agent to be used for downloaing podcasts.
        let agent = ureq::AgentBuilder::new()
            .redirects(8)
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();
        // If program state of previous episodes downloaded is available, store it so that we don't
        // download the same episodes again.
        let mut state = match get_state() {
            Ok(state) => state,
            Err(_) => HashMap::new()
        };
        let mut podcasts: HashMap<String, Podcast> = HashMap::new();
        for (podcast_id, podcast_url) in settings.podcasts.into_iter() {
            let no_files_downloaded: Vec<String> = Vec::new();
            let files_downloaded = state.remove(&podcast_id).unwrap_or(no_files_downloaded);
            let podcast = Podcast::new(podcast_id.clone(), podcast_url, &media_dir, files_downloaded, &agent);
            podcasts.insert(podcast_id, podcast);
        }
        Self { podcasts, player, playback_speed }
    }

    // Utility function to select podcasts based on the podcast ID. If no podcast ID is specified,
    // that means we are looking at all the podcasts from the settings.
    fn select_podcasts(&self, podcast_id: Option<String>) -> Vec<&Podcast> {
        match podcast_id {
            None => self.podcasts.values().collect(),
            Some(p_id) => self.podcasts.get(&p_id).into_iter().collect()
        }
    }
    
    // Mutable version of the above function. It is needed when we mutate the podcast obejcts
    // during "download" and "catchup" actions to mark the episodes as downloaded.
    fn select_podcasts_mut(&mut self, podcast_id: Option<String>) -> Vec<&mut Podcast> {
        match podcast_id {
            None => self.podcasts.values_mut().collect(),
            Some(p_id) => self.podcasts.get_mut(&p_id).into_iter().collect()
        }
    }

    // Handle download action.
    pub fn download(&mut self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts_mut(podcast_id);
        let count = count.unwrap_or(1);
        let progress = Arc::new(Mutex::new(Progress::new()));
        // Download the podcasts concurrently.

        for podcast in podcasts {
            let progress = Arc::clone(&progress);
            podcast.download(progress, count);
        }
        // At the end of the download, store the state - the list of episode
        // files downloaded for each podcast.
        let mut state: HashMap<&String, &Vec<String>> = HashMap::new();
        for (podcast_id, podcast) in self.podcasts.iter() {
            state.insert(podcast_id, &podcast.files_downloaded);
        }
        store_state(state).unwrap();
    }

    // Handle catchup action.
    pub fn catchup(&mut self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts_mut(podcast_id);
        let count = count.unwrap_or(5);
        for podcast in podcasts.into_iter() {
            podcast.catchup(count);
        }
    }

    // Handle list action.
    pub fn list(&self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(5);
        for podcast in podcasts.into_iter() {
            podcast.list(count);
        }
    }

    // Handle play action.
    pub fn play(&self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(1);
        let player = &self.player;
        let speed = self.playback_speed;
        // Create a playlist from latest downloaded episodes of the selected podcast(s).
        let playlist: Vec<&String> = podcasts.into_iter().flat_map(|podcast| {
            let episode_files: Vec<&String> = podcast.files_downloaded.iter().take(count).collect();
            episode_files.into_iter()
        }).collect();
        if playlist.is_empty() {
            println!("No episodes available to play; download them first.");
        } else {
            let child = Command::new(&player)
                .arg(format!("--rate={:.2}", speed))
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
