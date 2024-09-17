use std::str;
use std::fs;
use std::fs::File;
use std::thread;
use std::sync::Arc;
use url::Url;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use ureq;
use roxmltree;
use std::error::Error;
use crate::common::Episode;
use colored::Colorize;
use std::process::{Command, Stdio};
use crate::config::PodcastSetting;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};


// Parse the podcast feed to extract information of the episodes.
fn get_episodes(podcast_url: &String, count: usize) -> Result<Vec<Episode>, Box<dyn Error>> {
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
fn download_episode(agent: &ureq::Agent, url: &Url, path: &PathBuf, m: &Arc<MultiProgress>, sty: &ProgressStyle) -> Result<(), Box<dyn Error>> {
    let req = agent.request_url("GET", &url);
    let resp = req.call()?;
    let content_len: usize = resp.header("Content-Length").unwrap().parse()?;
    let plimit: u64 = u64::try_from(content_len).unwrap();
    let pb = m.add(ProgressBar::new(plimit));
    pb.set_style(sty.clone());
    pb.set_message(path.display().to_string());
    let mut bytes: Vec<u8> = Vec::with_capacity(content_len);
    resp.into_reader().read_to_end(&mut bytes)?;
    let mut file = File::create(&path)?;
    file.write_all(bytes.as_slice())?;
    pb.inc(plimit);
    pb.finish_with_message("done");
    Ok(())
}

fn download_podcast(agent: &ureq::Agent, podcast: PodcastSetting, media_dir: &str, count: usize, m: &Arc<MultiProgress>, sty: &ProgressStyle, previous_state: &HashMap<String, Vec<Episode>>) -> Result<Vec<Episode>, Box<dyn Error>> {
    let no_episodes: Vec<Episode> = Vec::new();
    let prev_downloaded_episodes = previous_state.get(&podcast.id).unwrap_or(&no_episodes);
    let dir_path = Path::new(media_dir).join(&podcast.id);
    let episodes = get_episodes(&podcast.url, count)?;
    fs::create_dir_all(&dir_path)?;
    let guids_downloaded: Vec<&str> = prev_downloaded_episodes.into_iter().map(|e| e.guid.as_str()).collect();
    let downloaded_episodes = episodes.into_iter()
        .filter(|episode| {
            if guids_downloaded.contains(&episode.guid.as_str()) {
                return false;
            }
            let url = &episode.url.as_str();
            let req_url: Url = match url.parse() {
                Ok(u) => u,
                Err(e) => {
                    eprintln!("{}: invalid episode URL {} ({:?})", &podcast.id.magenta().bold(), url, e);
                    return false;
                }
            };
            let file_name = Path::new(req_url.path()).file_name().unwrap();
            let path = dir_path.join(file_name);
            match download_episode(agent, &req_url, &path, m, sty) {
                Ok(()) => {
                    true
                },
                Err(e) => {
                    eprintln!("{}: error {:?} while downloading episode from {}", &podcast.id.magenta().bold(), e, &url);
                    false
                }
            }
        })
    .collect();
    Ok(downloaded_episodes)
}

pub fn download_podcasts(agent: &ureq::Agent, podcasts: Vec<PodcastSetting>, media_dir: &str, count: usize, previous_state: &HashMap<String, Vec<Episode>>) -> HashMap<String, Vec<Episode>> {
    thread::scope(|s| {
        let m = Arc::new(MultiProgress::new());
        let sty = ProgressStyle::with_template(
            "[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
        ).unwrap().progress_chars("##-");
        let handles: Vec<thread::ScopedJoinHandle<Option<(String, Vec<Episode>)>>> = podcasts.into_iter().map(|podcast| {
            let m = Arc::clone(&m);
            let podcast_id = podcast.id.clone();
            let sty = sty.clone();
            s.spawn(move || {
                match download_podcast(&agent, podcast, media_dir, count, &m, &sty, previous_state) {
                    Ok(downloaded_episodes) => {
                        m.println(format!("{} is downloaded!", &podcast_id)).unwrap();
                        Some((podcast_id, downloaded_episodes))
                    },
                    Err(e) => {
                        eprintln!("{}: Failed to download podcast: {:?}", &podcast_id.magenta().bold(), e);
                        None
                    }
                }
            })
        }).collect();
        let mut new_state: HashMap<String, Vec<Episode>> = HashMap::new();
        for handle in handles.into_iter() {
            match handle.join() {
                Ok(res) => {
                    res.and_then(|(podcast_id, episodes)| new_state.insert(podcast_id, episodes));
                },
                Err(e) => {
                    eprintln!("thread to download podcast failed: {:?}", e);
                }
            };
        }
        new_state
    })
}

pub fn compute_updated_state(new_state: HashMap<String, Vec<Episode>>, previous_state: HashMap<String, Vec<Episode>>) -> HashMap<String, Vec<Episode>> { 
    let no_episodes: Vec<Episode> = Vec::new();
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
    updated_state
}

pub fn show_remote(podcasts: Vec<PodcastSetting>, count: usize) {
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
}

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

