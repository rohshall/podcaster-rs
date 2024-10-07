use std::fmt;
use url::Url;
use std::io;
use std::path::PathBuf;
use std::error::Error;
use std::fs::File;
use colored::Colorize;
use ureq;
use std::sync::{Arc, Mutex};
use linya::{Bar, Progress};

#[derive(Debug)]
pub struct Episode {
    title: String,
    url: String,
    pub_date: String,
    pub file_name: String,
}

// Implement the trait Display to nicely show the episodes available in `show` action.
impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Truncate long titles to 74 characters.
        let mut title = String::from(&self.title);
        title.truncate(74);
        f.write_fmt(format_args!("{:80} {:30} ({})", title, self.file_name.yellow().bold(), self.pub_date))
    }
}

impl Episode {
    pub fn new(title: String, url: String, pub_date: String) -> Episode {
        // Use the file name, which is the last path segment of the URL.
        let req_url: Url = url.parse().unwrap();
        let file_name = req_url.path_segments().unwrap().last().unwrap().to_string();
        Self { title, url, pub_date, file_name }
    }

    // Download the episode from the URL.
    pub fn download(&self, agent: &ureq::Agent, dir_path: &PathBuf, progress: Arc<Mutex<Progress>>) -> Result<(), Box<dyn Error>> {
        let path = dir_path.join(&self.file_name);
        let req_url: Url = self.url.parse()?;
        let req = agent.request_url("GET", &req_url);
        let resp = req.call()?;
        let content_len: usize = resp.header("Content-Length").unwrap().parse()?;
        let bar: Bar = progress.lock().unwrap().bar(content_len, format!("Downloading {}", path.display().to_string()));
        let mut handle = File::create(&path)?;
        io::copy(&mut resp.into_reader(), &mut handle).unwrap();
        progress.lock().unwrap().inc_and_draw(&bar, content_len);
        Ok(())
    }
}
