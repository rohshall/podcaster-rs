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

pub fn get_episodes(podcast_url: &String, count: usize) -> Result<Vec<Episode>, Box<dyn Error>> {
    let podcast_response = attohttpc::get(podcast_url).send()?;
    let podcast_feed_contents = podcast_response.text()?;
    let podcast_feed_doc = roxmltree::Document::parse(&podcast_feed_contents)?;
    let episodes: Vec<Episode> = podcast_feed_doc.descendants()
        .filter(|n| n.has_tag_name("item"))
        .filter_map(|n| {
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
        println!("{:?}", episode);
        let url = Url::parse(episode.url.as_str())?;
        let file_name = Path::new(url.path()).file_name().unwrap();
        fs::create_dir_all(dir_path.to_str().unwrap())?;
        let path = dir_path.join(file_name);
        if path.exists() {
            println!("{:?} already downloaded", file_name);
        } else {
            println!("Downloading {:?} to {:?}", file_name, path);
            let mut resp = attohttpc::get(&episode.url).send()?;
            if resp.is_success() {
                let content_len: &str = resp.headers()["Content-Length"].to_str()?;
                let mut bytes: Vec<u8> = Vec::with_capacity(content_len.parse()?);
                resp.read_to_end(&mut bytes)?;
                let mut file = File::create(&path)?;
                file.write_all(bytes.as_slice())?;
            }
        }
    }
    Ok(())
}
