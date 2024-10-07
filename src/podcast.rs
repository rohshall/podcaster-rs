use std::vec::Vec;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{BufRead, BufReader, Write, BufWriter};
use std::fs::File;
use std::error::Error;
use std::sync::{Arc, Mutex};
use colored::Colorize;
use linya::Progress;
use ureq;
use roxmltree;
use crate::episode::Episode;

#[derive(Debug)]
pub struct Podcast {
    pub id: String,
    url: String,
    dir_path: PathBuf,
}

impl Podcast {
    pub fn new(id: String, url: String, media_dir: &str) -> Podcast {
        let dir_path = Path::new(media_dir).join(&id);
        fs::create_dir_all(&dir_path).expect("Failed to create directory for the podcast download");
        Self { id, url, dir_path }
    }

    // Fetch the podcast feed, and parse it to extract information of the episodes.
    fn fetch_episodes(&self, agent: &ureq::Agent, count: usize) -> Result<Vec<Episode>, Box<dyn Error>> {
        let podcast_response = agent.get(self.url.as_str()).call()?;
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

    // Read the previously downloaded list of episode files.
    fn get_files_downloaded(&self) -> Vec<String> {
        let state_path = &self.dir_path.join(".files");
        match File::open(&state_path) {
            Ok(f) => BufReader::new(f).lines().map(|line| line.unwrap()).collect(),
            Err(_) => Vec::new()
        }
    }

    // Store the current list of downloaded episode files.
    fn store_files_downloaded(&self, files_downloaded: Vec<String>) {
        let state_path = &self.dir_path.join(".files");
        match File::create(&state_path) {
            Ok(f) => {
                // Write at the most 20 most recent files to the state.
                // We maintain this state so that we don't download them again.
                let mut writer = BufWriter::new(f);
                for file in files_downloaded.into_iter().take(20) {
                    writer.write_all(&file.into_bytes()).unwrap();
                    writer.write_all("\n".as_bytes()).unwrap();
                }
                writer.flush().unwrap();
            },
            Err(e) => {
                eprintln!("{}: Failed to store the state of the downloads in {}; error {}", &self.id.magenta().bold(), &state_path.display(), e);
            }
        }
    }

    // Download the podcast by fetching the episodes and downloading the episodes that are not
    // already downloaded.
    pub fn download(&self, agent: &ureq::Agent, progress: Arc<Mutex<Progress>>, count: usize) {
        let episodes = match self.fetch_episodes(agent, count) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{}: error {:?} while fetching episodes", &self.id.magenta().bold(), e);
                return;
            }
        };
        let files_downloaded = self.get_files_downloaded();
        let new_files_downloaded: Vec<String> = episodes.into_iter().filter_map(|episode| {
            if files_downloaded.contains(&episode.file_name) {
                None
            } else {
                let progress = Arc::clone(&progress);
                match episode.download(agent, &self.dir_path, progress) {
                    Ok(()) => {
                        Some(episode.file_name.clone())
                    },
                    Err(e) => {
                        eprintln!("{}: error {:?} while downloading episode to {}", &self.id.magenta().bold(), e, &episode.file_name);
                        None
                    }
                }
            }
        }).collect();
        let files_downloaded = new_files_downloaded.into_iter().chain(files_downloaded).collect();
        self.store_files_downloaded(files_downloaded);
    }

    // Fetch the podcast episodes and mark the podcast episodes that are not already downloaded as downloaded.
    pub fn catchup(&self, agent: &ureq::Agent, count: usize) {
        let episodes = match self.fetch_episodes(agent, count) {
            Ok(e) => e,
            Err(e) => {
                eprintln!("{}: error {:?} while fetching episodes", &self.id.magenta().bold(), e);
                return;
            }
        };
        let files_downloaded = self.get_files_downloaded();
        let new_files_downloaded: Vec<String> = episodes.into_iter().filter_map(|episode| {
            if files_downloaded.contains(&episode.file_name) {
                None
            } else {
                Some(episode.file_name.clone())
            }
        }).collect();
        let files_downloaded = new_files_downloaded.into_iter().chain(files_downloaded).collect();
        self.store_files_downloaded(files_downloaded);
    }

    // List the podcast episodes. Mark the episodes that are yet to be downloaded with "*".
    pub fn list(&self, agent: &ureq::Agent, count: usize) {
        match self.fetch_episodes(agent, count) {
            Ok(episodes) => {
                let files_downloaded = self.get_files_downloaded();
                println!("\n{}:", &self.id.magenta().bold());
                // Indicate yet to be downloaded episodes with "*".
                for episode in episodes.iter() {
                    if files_downloaded.contains(&episode.file_name) {
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
