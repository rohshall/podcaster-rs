use std::path::Path;
use std::env::var;
use std::error::Error;
use std::collections::HashMap;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub media_dir: String,
    pub player: String,
    pub playback_speed: f64,
    pub podcasts: HashMap<String, String>,
}

impl Settings {
    pub fn new() -> Result<Settings, Box<dyn Error>> {
        let config_home = var("XDG_CONFIG_HOME")
            .or_else(|_| var("HOME")).unwrap();
        let config_file_path = Path::new(&config_home).join(".podcasts.toml");
        let contents = fs::read_to_string(config_file_path)?;
        let config: Settings = toml::from_str(&contents)?;
        Ok(config)
    }
}

