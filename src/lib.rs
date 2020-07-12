use ansi_term::Style;
use chrono::offset::Utc;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub struct ProcessedFeed {
    pub title: String,
    pub items: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Feed {
    pub uri: String,
    pub last_accessed: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigObj {
    pub feeds: Vec<Feed>,
}

pub fn find_config() -> std::path::PathBuf {
    let homedir: std::path::PathBuf = dirs::home_dir().expect("no home dir");
    //let path_to_config: &Path =
    Path::new(&homedir).join(".hemrc")
}

fn config_to_rust() -> ConfigObj {
    let config = fs::read_to_string(&find_config()).expect("reading config failed");
    serde_json::from_str(&config).unwrap()
}

pub fn rust_to_config(content: &[u8]) {
    let mut file = match File::create(find_config()) {
        Err(why) => panic!("config file access failed: {}", why),
        Ok(file) => file,
    };
    file.write_all(content)
        .expect("Writing to .hemrc failed :(");
}

pub fn add_feed(feed: &str) {
    let mut my_feeds: ConfigObj = config_to_rust();
    my_feeds.feeds.push(Feed {
        uri: feed.to_owned(),
        last_accessed: Utc::now().to_rfc3339().to_owned(),
    });
    rust_to_config(serde_json::to_string(&my_feeds).unwrap().as_bytes());
}

pub fn list_feeds() {
    let my_feeds: ConfigObj = config_to_rust();
    for f in my_feeds.feeds {
        println!("{}", f.uri);
    }
}

impl std::fmt::Display for ProcessedFeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "📖 {}\n\t{}",
            Style::new().bold().paint(&self.title),
            format!("{}", self.items.iter().format("\n\t"))
        )
    }
}
