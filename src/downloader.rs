use std::str;
use std::fs;
use std::fs::File;
use url::Url;
use std::io::Write;
use std::io::Read;
use std::path::{Path, PathBuf};
use ureq;
use roxmltree;

pub fn download_podcast(podcast_url: &String, dir_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let podcast_response = ureq::get(podcast_url).call()?;
    let podcast_feed_contents = podcast_response.into_string().unwrap();
    let podcast_feed_doc = roxmltree::Document::parse(&podcast_feed_contents)?;
    let enclosure_urls: Vec<&str> = podcast_feed_doc.descendants()
        .filter(|n| n.has_tag_name("enclosure"))
        .take(1)
        .map(|e| e.attribute("url").unwrap())
        .collect();
    let enclosure_url = match enclosure_urls.get(0) {
        Some(u) => u,
        None => {
            println!("No episode to download");
            return Ok(());
        }
    };

    //println!("{}", enclosure_url);
    let url = Url::parse(enclosure_url)?;
    let file_name = Path::new(url.path()).file_name().unwrap();
    fs::create_dir_all(dir_path.to_str().unwrap())?;
    let path = dir_path.join(file_name);
    println!("Downloading {:?} to {:?}", file_name, path);
    let resp = ureq::get(enclosure_url).call()?;
    let content_len: usize = resp.header("Content-Length").unwrap().parse()?;
    let mut bytes: Vec<u8> = Vec::with_capacity(content_len);
    resp.into_reader().read_to_end(&mut bytes)?;
    let mut file = File::create(&path)?;
    file.write_all(bytes.as_slice())?;
    Ok(())
}
