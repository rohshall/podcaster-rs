use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub title: String,
    pub guid: String,
    pub url: String,
    pub pub_date: String,
}

// Implement the trait Display to nicely show the episodes available in `show` action.
impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Truncate long titles to 74 characters.
        let mut title = String::from(&self.title);
        title.truncate(74);
        f.write_fmt(format_args!("{:80} ({})", title, self.pub_date))
    }
}
