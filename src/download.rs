use std::str;
use std::fs;
use std::fs::File;
use std::thread;
use std::thread::ScopedJoinHandle;
use std::sync::Arc;
use url::Url;
use std::io;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use ureq;
use roxmltree;
use std::error::Error;
use crate::common::Episode;
use colored::Colorize;
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
fn download_episode(agent: &ureq::Agent, url: &str, dir_path: &PathBuf, m: &Arc<MultiProgress>, sty: &ProgressStyle) -> Result<(), Box<dyn Error>> {
    let req_url: Url = url.parse()?;
    let file_name = Path::new(req_url.path()).file_name().unwrap();
    let path = dir_path.join(file_name);
    let req = agent.request_url("GET", &req_url);
    let resp = req.call()?;
    let content_len: usize = resp.header("Content-Length").unwrap().parse()?;
    let plimit: u64 = u64::try_from(content_len).unwrap();
    let pb = m.add(ProgressBar::new(plimit));
    pb.set_style(sty.clone());
    pb.set_message(path.display().to_string());
    let mut file = File::create(&path)?;
    io::copy(&mut pb.wrap_read(resp.into_reader()), &mut file).unwrap();
    Ok(())
}

fn get_podcast_episodes(podcasts: Vec<PodcastSetting>, media_dir: &str, count: usize, previous_state: &HashMap<String, Vec<Episode>>) -> Vec<(String, Vec<Episode>)> {
    podcasts.into_iter().filter_map(|podcast| 'podcast: {
        let no_episodes: Vec<Episode> = Vec::new();
        let prev_downloaded_episodes = previous_state.get(&podcast.id).unwrap_or(&no_episodes);
        let dir_path = Path::new(media_dir).join(&podcast.id);
        match fs::create_dir_all(&dir_path) {
            Ok(()) => {},
            Err(e) => {
                eprintln!("{}: Failed to create directory to download podcast: {:?}", &podcast.id.magenta().bold(), e);
                break 'podcast None
            }
        };
        let episodes = match get_episodes(&podcast.url, count) {
            Ok(episodes) => episodes,
            Err(e) => {
                eprintln!("{}: Failed to get episodes for the podcast: {:?}", &podcast.id.magenta().bold(), e);
                break 'podcast None
            }
        };
        let guids_downloaded: Vec<&str> = prev_downloaded_episodes.into_iter().map(|e| e.guid.as_str()).collect();
        let episodes_to_download = episodes.into_iter()
            .filter(|episode| !guids_downloaded.contains(&episode.guid.as_str()))
            .collect();
        Some((podcast.id.clone(), episodes_to_download))
    }).collect()
}

fn compute_new_state(handles: Vec<(String, Vec<ScopedJoinHandle<Option<Episode>>>)>) -> HashMap<String, Vec<Episode>> {
    let mut new_state: HashMap<String, Vec<Episode>> = HashMap::new();
    for (podcast_id, episode_handles) in handles.into_iter() {
        let downloaded_episodes = episode_handles.into_iter().filter_map(|handle| {
            match handle.join() {
                Ok(episode) => episode,
                Err(e) => {
                    eprintln!("thread to download podcast episode failed: {:?}", e);
                    None
                }
            }}).collect();
        new_state.insert(podcast_id.clone(), downloaded_episodes);
    }
    new_state
}

pub fn download_podcasts(agent: &ureq::Agent, podcasts: Vec<PodcastSetting>, media_dir: &str, count: usize, previous_state: HashMap<String, Vec<Episode>>) -> HashMap<String, Vec<Episode>> {
    let podcast_episodes = get_podcast_episodes(podcasts, media_dir, count, &previous_state);
    thread::scope(|s| {
        let m = Arc::new(MultiProgress::new());
        let sty = ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})i {msg}",
        ).unwrap().progress_chars("##-");
        let handles = podcast_episodes.into_iter().map(|(podcast_id, episodes)| {
            let episode_handles = episodes.into_iter().map(|episode| {
                let m = Arc::clone(&m);
                let sty = sty.clone();
                let podcast_id = podcast_id.clone();
                s.spawn(move || {
                    let dir_path = Path::new(media_dir).join(&podcast_id);
                    match download_episode(agent, &episode.url.as_str(), &dir_path, &m, &sty) {
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
            (podcast_id, episode_handles)
        }).collect();
        let new_state = compute_new_state(handles);
        compute_updated_state(new_state, previous_state)
    })
}

fn compute_updated_state(new_state: HashMap<String, Vec<Episode>>, previous_state: HashMap<String, Vec<Episode>>) -> HashMap<String, Vec<Episode>> { 
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

