use std::str;
use std::fs;
use std::fs::File;
use url::Url;
use std::io::Write;
use std::io::Read;
use std::path::{Path, PathBuf};
use attohttpc;
use roxmltree;
use std::error::Error;
use crate::common::Episode;

// Parse the podcast feed to extract information of the episodes.
pub fn get_episodes(podcast_url: &String, count: usize) -> Result<Vec<Episode>, Box<dyn Error>> {
    let podcast_response = attohttpc::get(podcast_url).send()?;
    let podcast_feed_contents = podcast_response.text()?;
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

// Utility function to download from an url to a file, called from the download_episode function.
fn download_url(url: &Url, path: &Path) -> Result<(), Box<dyn Error>> { 
    // Some podcast episode URLs need too many redirections.
    let mut resp = attohttpc::get(url).max_redirections(8).send()?;
    if resp.is_success() {
        println!("Downloading {:?}", path);
        let content_len: &str = resp.headers()["Content-Length"].to_str()?;
        let mut bytes: Vec<u8> = Vec::with_capacity(content_len.parse()?);
        resp.read_to_end(&mut bytes)?;
        let mut file = File::create(&path)?;
        file.write_all(bytes.as_slice())?;
        Ok(())
    } else {
        Err(Box::from(format!("Failed to get valid response from {:?}", url)))
    }
}

// Download the episode from the URL.
fn download_episode(url: &str, dir_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    let url = Url::parse(url)?;
    let file_name = Path::new(url.path()).file_name().unwrap();
    let path = dir_path.join(file_name);
    download_url(&url, &path)?;
    Ok(())
}

// Try to download all episodes, and return the list of downloaded episodes.
pub fn download_podcast(episodes: Vec<Episode>, dir_path: &PathBuf, episodes_downloaded: &Vec<Episode>) -> Result<Vec<Episode>, Box<dyn Error>> {
    fs::create_dir_all(dir_path)?;
    let guids_downloaded: Vec<&str> = episodes_downloaded.into_iter().map(|e| e.guid.as_str()).collect();
    let downloaded = episodes.into_iter()
        .filter(|episode| {
            if guids_downloaded.contains(&episode.guid.as_str()) {
                return false;
            }
            match download_episode(&episode.url.as_str(), dir_path) {
                Ok(()) => true,
                Err(e) => {
                    println!("Error {:?} while downloading episode from {}", e, &episode.url);
                    false
                }
            }
        })
    .collect();
    Ok(downloaded)
}

<<<<<<< Updated upstream
pub fn get_latest_episode_download(episode: Episode, dir_path: &PathBuf) -> Option<PathBuf> {
=======
pub fn get_episode_download(episode: &Episode, dir_path: &PathBuf) -> Option<PathBuf> {
>>>>>>> Stashed changes
    Url::parse(&episode.url.as_str()).ok().and_then(|url| -> Option<PathBuf> {
        let file_name = Path::new(url.path()).file_name().unwrap();
        let path = dir_path.join(file_name);
        match path.try_exists() {
            Ok(exists) => exists.then_some(path),
            Err(_) => None
        }
    })
}
