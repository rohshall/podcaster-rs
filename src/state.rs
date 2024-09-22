use std::path::Path;
use std::env::var;
use std::vec::Vec;
use std::fs::File;
use std::error::Error;
use std::collections::HashMap;
use std::fs;
use std::io::Write;


// Get the stored state of episodes downloaded for each podcast. This is done at the program
// start-up.
pub fn get_state() -> Result<HashMap<String, Vec<String>>, Box<dyn Error>> {
    let state_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME")).unwrap();
    let state_file_path = Path::new(&state_home).join(".podcaster_state.json");
    let contents = fs::read_to_string(state_file_path)?;
    let state: HashMap<String, Vec<String>> = serde_json::from_str(&contents)?;
    Ok(state)
}

// Store the state of episodes downloaded for each podcast. This is done when the program
// terminates.
pub fn store_state(state_contents: HashMap<&String, &Vec<String>>) -> Result<(), Box<dyn Error>> {
    let state_home = var("XDG_CONFIG_HOME")
        .or_else(|_| var("HOME")).unwrap();
    let state_file_path = Path::new(&state_home).join(".podcaster_state.json");
    let mut state_file = File::create(&state_file_path)?;
    let state_contents = serde_json::to_string_pretty(&state_contents)?;
    state_file.write_all(state_contents.as_bytes())?;
    Ok(())
}
