use std::error::Error;
use std::collections::HashMap;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub media_dir: String,
    pub player: String,
    pub podcasts: HashMap<String, String>,
}

impl Settings {
    pub fn new() -> Result<Settings, Box<dyn Error>> {
        let mut config_path = dirs::config_dir().unwrap();
        config_path.push("podcaster-rs");
        config_path.push("podcasts.toml");
        let contents = fs::read_to_string(config_path)?;
        let config: Settings = toml::from_str(&contents)?;
        Ok(config)
    }
}

