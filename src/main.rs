use std::str;
use std::collections::HashMap;
use serde::Deserialize;
use std::fs;
use std::fs::File;
use url::Url;
use std::io::Write;
use std::io::Read;
use std::path::Path;
use std::env;
use ureq;
use roxmltree;

#[derive(Debug, Deserialize)]
struct Data {
    media_dir: String,
    podcasts: HashMap<String, String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let podcast_id = &args[1];

    let filename = "/home/salil/.podcasts.toml";

    let contents = fs::read_to_string(filename).unwrap();
    let data: Data = toml::from_str(&contents).unwrap();

    let url = &data.podcasts[podcast_id];
    let body = ureq::get(url).call().unwrap().into_string().unwrap();
    let doc = roxmltree::Document::parse(&body).unwrap();
    let latest_episode_url = doc.descendants().find(|n| n.has_tag_name("enclosure")).unwrap().attribute("url").unwrap();
    //println!("{}", latest_episode_url);
    let url = Url::parse(latest_episode_url).unwrap();
    let file_name = Path::new(url.path()).file_name().unwrap();
    let dir_path = Path::new(&data.media_dir).join(podcast_id);
    fs::create_dir_all(dir_path.to_str().unwrap()).unwrap();
    let path = dir_path.join(file_name);
    println!("Downloading {:?} to {:?}", file_name, path);
    let resp = ureq::get(latest_episode_url).call().unwrap();
    let len: usize = resp.header("Content-Length").unwrap().parse().unwrap();
    let mut bytes: Vec<u8> = Vec::with_capacity(len);
    resp.into_reader().read_to_end(&mut bytes).unwrap();
    
    let mut file = File::create(&path).unwrap(); 
    file.write_all(bytes.as_slice()).unwrap();
}

