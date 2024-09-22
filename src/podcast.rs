use std::vec::Vec;
use std::path::{Path, PathBuf};
use std::fs;
use std::error::Error;
use std::sync::{Arc, Mutex};
use colored::Colorize;
use linya::Progress;
use ureq;
use roxmltree;
use crate::episode::Episode;

#[derive(Debug)]
pub struct Podcast {
    id: String,
    url: String,
    dir_path: PathBuf,
    // Store the list of the episodes already downloaded. During the program start-up, we populate
    // this from the state stored. And after the download, we update the stored state.
    pub files_downloaded: Vec<String>,
    agent: ureq::Agent,
}

impl Podcast {
    pub fn new(id: String, url: String, media_dir: &str, files_downloaded: Vec<String>, agent: &ureq::Agent) -> Podcast {
        let dir_path = Path::new(media_dir).join(&id);
        fs::create_dir_all(&dir_path).expect("Failed to create directory for the podcast download");
        let agent = agent.clone();
        Self { id, url, dir_path, files_downloaded, agent }
    }

    // Fetch the podcast feed, and parse it to extract information of the episodes.
    fn fetch_episodes(&self, count: usize) -> Result<Vec<Episode>, Box<dyn Error>> {
        let podcast_response = self.agent.get(self.url.as_str()).call()?;
        let podcast_feed_contents = podcast_response.into_string()?;
        let podcast_feed_doc = roxmltree::Document::parse(&podcast_feed_contents)?;
        let episodes: Vec<Episode> = podcast_feed_doc.descendants()
            .filter(|n| n.has_tag_name("item"))
            .filter_map(|n| {
                // Only look for "audio/mpeg" enclosures in the podcast feed to get the episodes.
                if let Some(enclosure) = n.children().find(|e| e.has_tag_name("enclosure") && e.attribute("type").unwrap() == "audio/mpeg") {
                    let title = n.children().find(|t| t.has_tag_name("title")).unwrap().text().unwrap().to_string();
                    let pub_date = n.children().find(|p| p.has_tag_name("pubDate")).unwrap().text().unwrap().to_string();
                    let url = enclosure.attribute("url").unwrap().to_string();
                    let episode = Episode::new(title, url, pub_date);
                    Some(episode)
                } else {
                    None
                }
            }).take(count)
        .collect();
        Ok(episodes)
    }

    // Download the podcast by fetching the episodes and downloading the episodes that are not
    // already downloaded.
    pub fn download(&mut self, progress: Arc<Mutex<Progress>>, count: usize) {
        let episodes = match self.fetch_episodes(count) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{}: error {:?} while fetching episodes", &self.id.magenta().bold(), e);
                return;
            }
        };
        for episode in episodes.into_iter() {
            if self.files_downloaded.contains(&episode.file_name) {
                continue;
            }
            let progress = Arc::clone(&progress);
            match episode.download(&self.agent, &self.dir_path, &progress) {
                Ok(()) => {
                    self.files_downloaded.push(episode.file_name.clone());
                },
                Err(e) => {
                    eprintln!("{}: error {:?} while downloading episode to {}", &self.id.magenta().bold(), e, &episode.file_name);
                }
            }
        }
    }

    // Fetch the podcast episodes and mark the podcast episodes that are not already downloaded as downloaded.
    pub fn catchup(&mut self, count: usize) {
        let episodes = match self.fetch_episodes(count) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{}: error {:?} while fetching episodes", &self.id.magenta().bold(), e);
                return;
            }
        };
        for episode in episodes.into_iter() {
            if self.files_downloaded.contains(&episode.file_name) {
                continue;
            }
            self.files_downloaded.push(episode.file_name.clone());
        }
    }

    // List the podcast episodes. Mark the episodes that are yet to be downloaded with "*".
    pub fn list(&self, count: usize) {
        match self.fetch_episodes(count) {
            Ok(episodes) => {
                println!("\n{}:", &self.id.magenta().bold());
                // Indicate yet to be downloaded episodes with "*".
                for episode in episodes.iter() {
                    if self.files_downloaded.contains(&episode.file_name) {
                        println!(" {}", episode);
                    } else {
                        println!("{}{}", "*".yellow().bold(), episode);
                    }
                }
            },
            Err(e) => {
                eprintln!("Could not get the podcast feed: {}", e);
            },
        }
    }
}
