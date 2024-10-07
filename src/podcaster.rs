use std::time::Duration;
use std::thread;
use std::thread::ScopedJoinHandle;
use std::vec::Vec;
use std::collections::HashMap;
use ureq;
use std::sync::{Arc, Mutex};
use linya::Progress;
use crate::settings::Settings;
use crate::podcast::Podcast;

#[derive(Debug)]
pub struct Podcaster {
    // Model podcasts as a HashMap to quickly find out the podcast based on the podcast ID.
    podcasts: HashMap<String, Podcast>,
    agent: ureq::Agent,
}

impl Podcaster {
    pub fn new() -> Podcaster {
        let settings = Settings::new().expect("Failed to parse the config file");
        // Store the config params.
        let media_dir = settings.media_dir;
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
        Self { podcasts, agent }
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

}
