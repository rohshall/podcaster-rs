use std::fmt;
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

#[derive(Debug)]
pub struct Episode {
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
        })
    .take(count)
        .collect();
    Ok(episodes)
}

// Utility function to download from an url to a file, called from the download_episode function.
fn download_url(url: &Url, file_name: &str, path: &Path) -> Result<(), Box<dyn Error>> { 
    // Some podcast episode URLs need too many redirections.
    let mut resp = attohttpc::get(url).max_redirections(8).send()?;
    if resp.is_success() {
        println!("Downloading {:?} to {:?}", file_name, path);
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

// While downloading an episode, print errors, and return whether it was downloaded or not.
fn download_episode(episode: &Episode, dir_path: &PathBuf) -> bool {
    match Url::parse(episode.url.as_str()) {
        Err(e) => {
            println!("Invalid episode download URL {:?} ({:?})", episode.url, e);
            false
        },
        Ok(url) => {
            let file_name = Path::new(url.path()).file_name().unwrap();
            let path = dir_path.join(file_name);
            match download_url(&url, file_name.to_str().unwrap(), &path) {
                Ok(()) => true,
                Err(e) => {
                    println!("Failed to download {:?} to {:?} ({:?})", url, file_name, e);
                    false
                }
            }
        }
    }
}

// Try to download all episodes, and return the list of downloaded episodes.
pub fn download_podcast(episodes: Vec<Episode>, dir_path: &PathBuf, episodes_downloaded: &Vec<String>) -> Result<Vec<Episode>, Box<dyn Error>> {
    fs::create_dir_all(dir_path)?;
    let downloaded = episodes.into_iter()
        .filter(|episode| episodes_downloaded.contains(&episode.guid) || download_episode(&episode, dir_path))
        .collect();
    Ok(downloaded)
}
