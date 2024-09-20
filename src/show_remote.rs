use ureq;
use roxmltree;
use std::error::Error;
use crate::common::Episode;
use colored::Colorize;
use crate::config::PodcastSetting;


// Parse the podcast feed to extract information of the episodes.
fn get_episodes(podcast_url: &String, count: usize) -> Result<Vec<Episode>, Box<dyn Error>> {
    let podcast_response = ureq::get(podcast_url).call()?;
    let podcast_feed_contents = podcast_response.into_string()?;
    let podcast_feed_doc = roxmltree::Document::parse(&podcast_feed_contents)?;
    let episodes: Vec<Episode> = podcast_feed_doc.descendants()
        .filter(|n| n.has_tag_name("item"))
        .filter_map(|n| {
            // Only look for "audio/mpeg" enclosures in the podcast feed to get the episodes.
            if let Some(enclosure) = n.children().find(|e| e.has_tag_name("enclosure") && e.attribute("type").unwrap() == "audio/mpeg") {
                let title = n.children().find(|t| t.has_tag_name("title")).unwrap().text().unwrap();
                let guid = n.children().find(|g| g.has_tag_name("guid")).unwrap().text().unwrap();
                let pub_date = n.children().find(|p| p.has_tag_name("pubDate")).unwrap().text().unwrap();
                let url = enclosure.attribute("url").unwrap();
                let episode = Episode { title: String::from(title), guid: String::from(guid), url: String::from(url), pub_date: String::from(pub_date)};
                Some(episode)
            } else {
                None
            }
        }).take(count)
    .collect();
    Ok(episodes)
}

pub fn show_remote(podcasts: Vec<PodcastSetting>, count: usize) {
    for podcast in podcasts.into_iter() {
        match get_episodes(&podcast.url, count) {
            Ok(episodes) => {
                println!("\n{}:", podcast.id.magenta().bold());
                for episode in episodes.iter() {
                    println!("{}", episode);
                }
            },
            Err(e) => {
                eprintln!("Could not get the podcast feed: {}", e);
            },
        }
    }
}
