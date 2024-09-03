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

pub fn download_podcast(podcast_url: &String, dir_path: &PathBuf, count: usize) -> Result<(), Box<dyn Error>> {
    let podcast_response = attohttpc::get(podcast_url).send()?;
    let podcast_feed_contents = podcast_response.text()?;
    let podcast_feed_doc = roxmltree::Document::parse(&podcast_feed_contents)?;
    let enclosure_urls: Vec<&str> = podcast_feed_doc.descendants()
        .filter(|n| n.has_tag_name("enclosure"))
        .filter(|e| e.attribute("type").unwrap() == "audio/mpeg")
        .map(|e| e.attribute("url").unwrap())
        .collect();

    for enclosure_url in enclosure_urls.iter().take(count) {

        //println!("{}", enclosure_url);
        let url = Url::parse(enclosure_url)?;
        let file_name = Path::new(url.path()).file_name().unwrap();
        fs::create_dir_all(dir_path.to_str().unwrap())?;
        let path = dir_path.join(file_name);
        println!("Downloading {:?} to {:?}", file_name, path);
        let mut resp = attohttpc::get(enclosure_url).send()?;
        if resp.is_success() {
            let content_len: &str = resp.headers()["Content-Length"].to_str()?;
            let mut bytes: Vec<u8> = Vec::with_capacity(content_len.parse()?);
            resp.read_to_end(&mut bytes)?;
            let mut file = File::create(&path)?;
            file.write_all(bytes.as_slice())?;
        }
    }
    Ok(())

}
