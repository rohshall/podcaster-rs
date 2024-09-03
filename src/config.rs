use std::path::Path;
use std::env::var;
use std::fs;
use std::vec::Vec;
use serde::Deserialize;
use std::error::Error;

#[derive(Debug, Deserialize)]
pub struct Podcast {
    pub id: String,
    pub url: String,
}
#[derive(Debug, Deserialize)]
pub struct Config {
    pub media_dir: String
}
#[derive(Debug, Deserialize)]
pub struct Settings {
    pub config: Config,
    pub podcasts: Vec<Podcast>,
}

pub fn get_config() -> Result<Settings, Box<dyn Error>> {
    let config_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME")).unwrap();
    let config_file_name = Path::new(&config_home).join(".podcasts.json");
    let contents = fs::read_to_string(config_file_name)?;
    let config: Settings = serde_json::from_str(&contents)?;
    Ok(config)
}
