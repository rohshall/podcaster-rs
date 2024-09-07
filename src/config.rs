use std::path::Path;
use std::fs::File;
use std::io::Write;
use std::env::var;
use std::fs;
use std::vec::Vec;
use serde::Deserialize;
use std::error::Error;
use std::collections::HashMap;

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
    let config_file_path = Path::new(&config_home).join(".podcasts.json");
    let contents = fs::read_to_string(config_file_path)?;
    let config: Settings = serde_json::from_str(&contents)?;
    Ok(config)
}

pub fn get_state() -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    let state_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME")).unwrap();
    let state_file_path = Path::new(&state_home).join(".podcaster_state.json");
    let contents = fs::read_to_string(state_file_path)?;
    let state: HashMap<String, Vec<String>> = serde_json::from_str(&contents)?;
    Ok(state)
}

pub fn store_state(state_contents: HashMap<String, Vec<String>>) -> Result<(), Box<dyn Error>> {
    let state_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME")).unwrap();
    let state_file_path = Path::new(&state_home).join(".podcaster_state.json");
    let mut state_file = File::create(&state_file_path)?;
    let state_contents = serde_json::to_string_pretty(&state_contents)?;
    state_file.write_all(state_contents.as_bytes())?;
    Ok(())
}
