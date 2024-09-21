use std::fmt;
use url::Url;
use std::io;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::time::Duration;
use std::env::var;
use std::vec::Vec;
use std::error::Error;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs;
use std::fs::File;
use ureq;
use roxmltree;
use std::thread;
use std::thread::ScopedJoinHandle;
use std::sync::{Arc, Mutex};
use colored::Colorize;
use linya::{Bar, Progress};
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Episode {
    pub title: String,
    pub guid: String,
    pub url: String,
    pub pub_date: String,
}

// Implement the trait Display to nicely show the episodes available in `show` action.
impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Truncate long titles to 74 characters.
        let mut title = String::from(&self.title);
        title.truncate(74);
        f.write_fmt(format_args!("{:80} ({})", title, self.pub_date))
    }
}

#[derive(Debug, Deserialize)]
struct Settings {
    pub media_dir: String,
    pub player: String,
    pub playback_speed: f64,
    pub podcasts: HashMap<String, String>,
}

fn get_settings() -> Result<Settings, Box<dyn Error>> {
    let config_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME")).unwrap();
    let config_file_path = Path::new(&config_home).join(".podcasts.toml");
    let contents = fs::read_to_string(config_file_path)?;
    let config: Settings = toml::from_str(&contents)?;
    Ok(config)
}

fn get_state() -> Result<HashMap<String, Vec<Episode>>, Box<dyn Error>> {
    let state_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME")).unwrap();
    let state_file_path = Path::new(&state_home).join(".podcaster_state.json");
    let contents = fs::read_to_string(state_file_path)?;
    let state: HashMap<String, Vec<Episode>> = serde_json::from_str(&contents)?;
    Ok(state)
}

fn store_state(state_contents: &HashMap<String, Vec<Episode>>) -> Result<(), Box<dyn Error>> {
    let state_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME")).unwrap();
    let state_file_path = Path::new(&state_home).join(".podcaster_state.json");
    let mut state_file = File::create(&state_file_path)?;
    let state_contents = serde_json::to_string_pretty(&state_contents)?;
    state_file.write_all(state_contents.as_bytes())?;
    Ok(())
}

// Parse the podcast feed to extract information of the episodes.
fn get_episodes(podcast_url: &str, count: usize) -> Result<Vec<Episode>, Box<dyn Error>> {
    let podcast_response = ureq::get(podcast_url).call()?;
    let podcast_feed_contents = podcast_response.into_string()?;
    let podcast_feed_doc = roxmltree::Document::parse(&podcast_feed_contents)?;
    let episodes: Vec<Episode> = podcast_feed_doc.descendants()
        .filter(|n| n.has_tag_name("item"))
        .filter_map(|n| {
            // Only look for "audio/mpeg" enclosures in the podcast feed to get the episodes.
            if let Some(enclosure) = n.children().find(|e| e.has_tag_name("enclosure") && e.attribute("type").unwrap() == "audio/mpeg") {
                let title = n.children().find(|t| t.has_tag_name("title")).unwrap().text().unwrap();
                let guid = n.children().find(|g| g.has_tag_name("guid")).unwrap().text().unwrap();
                let pub_date = n.children().find(|p| p.has_tag_name("pubDate")).unwrap().text().unwrap();
                let url = enclosure.attribute("url").unwrap();
                let episode = Episode { title: String::from(title), guid: String::from(guid), url: String::from(url), pub_date: String::from(pub_date)};
                Some(episode)
            } else {
                None
            }
        }).take(count)
    .collect();
    Ok(episodes)
}

// Download the episode from the URL.
fn download_episode(agent: &ureq::Agent, url: &str, dir_path: &PathBuf, progress: &Arc<Mutex<Progress>>) -> Result<(), Box<dyn Error>> {
    let req_url: Url = url.parse()?;
    let file_name = Path::new(req_url.path()).file_name().unwrap();
    let path = dir_path.join(file_name);
    let req = agent.request_url("GET", &req_url);
    let resp = req.call()?;
    let content_len: usize = resp.header("Content-Length").unwrap().parse()?;
    let bar: Bar = progress.lock().unwrap().bar(content_len, format!("Downloading {}", path.display()));
    let mut handle = File::create(&path)?;
    io::copy(&mut resp.into_reader(), &mut handle).unwrap();
    progress.lock().unwrap().inc_and_draw(&bar, content_len);
    Ok(())
}

fn compute_new_state(handles: Vec<(String, Vec<ScopedJoinHandle<Option<Episode>>>)>) -> Vec<(String, Vec<Episode>)> {
    handles.into_iter().map(|(podcast_id, episode_handles)| {
        let downloaded_episodes = episode_handles.into_iter().filter_map(|handle| {
            match handle.join() {
                Ok(episode) => episode,
                Err(e) => {
                    eprintln!("thread to download podcast episode failed: {:?}", e);
                    None
                }
            }}).collect();
        (podcast_id, downloaded_episodes)
    }).collect()
}

fn update_state(state: &mut HashMap<String, Vec<Episode>>, new_state: Vec<(String, Vec<Episode>)>) { 
    let no_episodes: Vec<Episode> = Vec::new();
    // Merge the previous state into the new state to get the updated current state.
    // App state consists of what episodes were downloaded for what podcasts.
    // To avoid storing infinite history, truncate it to latest 100 episodes.
    for (podcast_id, new_episodes) in new_state.into_iter() {
        let mut new_episodes = new_episodes;
        let previous_episodes = state.get(&podcast_id).unwrap_or(&no_episodes);
        new_episodes.extend_from_slice(previous_episodes.as_slice());
        state.insert(podcast_id.clone(), new_episodes);
    }
}

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

#[derive(Debug)]
pub struct Podcaster {
    settings: Settings,
    state: HashMap<String, Vec<Episode>>,
    agent: ureq::Agent,
}

impl Podcaster {
    pub fn new() -> Podcaster {
        let settings = get_settings().expect("Failed to parse the config file");
        let state = match get_state() {
            Ok(state) => state,
            Err(_) => HashMap::new()
        };
        let agent = ureq::AgentBuilder::new()
            .redirects(8)
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();
        Self { settings, state, agent }
    }

    // Utility function to select podcasts based on the podcast ID. If no podcast ID is specified,
    // that means we are looking at all the podcasts from the settings.
    fn select_podcasts(&self, podcast_id: Option<String>) -> Vec<(&String, &String)> {
        match podcast_id {
            None => self.settings.podcasts.iter().collect(),
            Some(p_id) => self.settings.podcasts.get_key_value(&p_id).into_iter().collect()
        }
    }

    fn get_podcast_episodes(&self, podcasts: Vec<(&String, &String)>, count: usize) -> Vec<(String, Vec<Episode>)> {
        podcasts.into_iter().filter_map(|(podcast_id, podcast_url)| 'podcast: {
            let no_episodes: Vec<Episode> = Vec::new();
            let prev_downloaded_episodes = self.state.get(podcast_id).unwrap_or(&no_episodes);
            let dir_path = Path::new(&self.settings.media_dir).join(podcast_id);
            match fs::create_dir_all(&dir_path) {
                Ok(()) => {},
                Err(e) => {
                    eprintln!("{}: Failed to create directory to download podcast: {:?}", podcast_id.magenta().bold(), e);
                    break 'podcast None
                }
            };
            let episodes = match get_episodes(podcast_url, count) {
                Ok(episodes) => episodes,
                Err(e) => {
                    eprintln!("{}: Failed to get episodes for the podcast: {:?}", podcast_id.magenta().bold(), e);
                    break 'podcast None
                }
            };
            // Do not download already downloaded episodes.
            let guids_downloaded: Vec<&str> = prev_downloaded_episodes.into_iter().map(|e| e.guid.as_str()).collect();
            let episodes_to_download: Vec<Episode> = episodes.into_iter()
                .filter(|episode| !guids_downloaded.contains(&episode.guid.as_str()))
                .collect();
            if episodes_to_download.is_empty() {
                None
            } else {
                Some((podcast_id.clone(), episodes_to_download))
            }
        }).collect()
    }

    fn download_helper(&self, podcast_episodes: Vec<(String, Vec<Episode>)>) -> Result<Vec<(String, Vec<Episode>)>, Box<dyn Error>> {
        thread::scope(|s| {
            let progress = Arc::new(Mutex::new(Progress::new()));
            // Download the episodes concurrently using threads.
            let handles = podcast_episodes.into_iter().map(|(podcast_id, episodes)| {
                let episode_handles = episodes.into_iter().map(|episode| {
                    let progress = Arc::clone(&progress);
                    let podcast_id = podcast_id.clone();
                    let dir_path = Path::new(&self.settings.media_dir).join(&podcast_id);
                    // Use scoped thread so that the closure can borrow non-static variables.
                    s.spawn(move || {
                        match download_episode(&self.agent, &episode.url.as_str(), &dir_path, &progress) {
                            Ok(()) => {
                                Some(episode)
                            },
                            Err(e) => {
                                eprintln!("{}: error {:?} while downloading episode from {}", &podcast_id.magenta().bold(), e, &episode.url.as_str());
                                None
                            }
                        }
                    })
                }).collect();
                (podcast_id.clone(), episode_handles)
            }).collect();
            Ok(compute_new_state(handles))
        })
    }

    pub fn download(&mut self, podcast_id: Option<String>, count: Option<usize>) -> Result<(), Box<dyn Error>> {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(1);
        // Collect all the episodes to be downloaded.
        let podcast_episodes = self.get_podcast_episodes(podcasts, count);
        // Download them.
        let new_state = self.download_helper(podcast_episodes)?;
        update_state(&mut self.state, new_state);
        Ok(())
    }

    pub fn catchup(&mut self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(5);
        let podcast_episodes = self.get_podcast_episodes(podcasts, count);
        // Mark all the episodes as downloaded.
        update_state(&mut self.state, podcast_episodes);
    }

    pub fn list(&self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(5);
        let no_episodes: Vec<Episode> = Vec::new();
        for (podcast_id, podcast_url) in podcasts.into_iter() {
            // Get the GUIDs of the episodes already downloaded from the state.
            let downloaded_episodes: Vec<String> = self.state.get(podcast_id).unwrap_or(&no_episodes).into_iter().map(|e| e.guid.clone()).collect();
            match get_episodes(podcast_url, count) {
                Ok(episodes) => {
                    println!("\n{}:", podcast_id.magenta().bold());
                    // Indicate yet to be downloaded episodes with "*".
                    for episode in episodes.iter() {
                        if downloaded_episodes.contains(&episode.guid) {
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

    pub fn play(&self, podcast_id: Option<String>, count: Option<usize>) {
        let podcasts = self.select_podcasts(podcast_id);
        let count = count.unwrap_or(1);
        let no_episodes: Vec<Episode> = Vec::new();
        let media_dir = Path::new(&self.settings.media_dir);
        let player = &self.settings.player;
        let speed = self.settings.playback_speed;
        // Create a playlist from latest downloaded episodes of the selected podcast(s).
        let playlist: Vec<PathBuf> = podcasts.into_iter().flat_map(|(podcast_id, _)| {
            let episodes: Vec<&Episode> = self.state.get(podcast_id).unwrap_or(&no_episodes).iter().take(count).collect();
            let dir_path = media_dir.join(podcast_id);
            let episode_files: Vec<PathBuf> = episodes.into_iter().filter_map(|episode| get_episode_download(episode, &dir_path)).collect();
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

impl Drop for Podcaster {
    fn drop(&mut self) {
        match store_state(&self.state) {
            Ok(()) => {},
            Err(e) => eprintln!("Error storing the podcaster state: {:?}", e)
        }
    }
}
