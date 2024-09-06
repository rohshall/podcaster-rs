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
    title: String,
    guid: String,
    url: String,
    pub_date: String,
}

impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Truncate long titles to 74 characters.
        let mut title = String::from(&self.title);
        title.truncate(74);
        f.write_fmt(format_args!("{:80} ({})", title, self.pub_date))
    }
}

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

pub fn download_podcast(episodes: &Vec<Episode>, dir_path: &PathBuf) -> Result<(), Box<dyn Error>> {
    for episode in episodes.iter() {
        let url = Url::parse(episode.url.as_str())?;
        let file_name = Path::new(url.path()).file_name().unwrap();
        fs::create_dir_all(dir_path)?;
        let path = dir_path.join(file_name);
        if path.exists() {
            println!("{:?} already downloaded", file_name);
        } else {
            // Some podcast episode URLs need too many redirections.
            match attohttpc::get(&episode.url).max_redirections(8).send() {
                Ok(mut resp) => {
                    if resp.is_success() {
                        println!("Downloading {:?} to {:?}", file_name, path);
                        let content_len: &str = resp.headers()["Content-Length"].to_str()?;
                        let mut bytes: Vec<u8> = Vec::with_capacity(content_len.parse()?);
                        resp.read_to_end(&mut bytes)?;
                        let mut file = File::create(&path)?;
                        file.write_all(bytes.as_slice())?;
                    } else {
                        println!("Failed to download {:?} to {:?}, received status {:?}", file_name, path, resp.status());
                    }
                },
                Err(e) => {
                    println!("Failed to get the episode from {:?} due to {:?}", episode.url, e);
                }
            }
        }
    }
    Ok(())
}
