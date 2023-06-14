use std::path::Path;
use std::env::var;
use std::fs;
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub media_dir: String,
    pub podcasts: HashMap<String, String>,
}

pub fn get_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME").map(|home|format!("{}/.config/podcaster", home)))?;
    let config_file_name = Path::new(&config_home).join(".podcasts.toml");
    let contents = fs::read_to_string(config_file_name)?;
    let config = toml::from_str(&contents).unwrap();
    Ok(config)
}
