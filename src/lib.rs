use ansi_term::Style;
use anyhow::{Context, Result};
use chrono::offset::Utc;
use chrono::DateTime;
use dialoguer::{theme::SimpleTheme, MultiSelect};
use feed_rs::parser;
use futures::stream::StreamExt;
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct ProcessedFeed {
    pub title: String,
    pub items: Vec<String>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

fn config_to_rust() -> Result<ConfigObj, Box<dyn std::error::Error>> {
    let config_path = find_config();
    let config = match fs::read_to_string(&config_path) {
        Ok(config) => config,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("Didn't find a .hemrc, creating it now...");
                // create the file and populate it with an empty array
                let mut configfile = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(&config_path)?;
                configfile.write_all(r#"{"feeds": []}"#.as_bytes())?;
                let contents = String::from(r#"{"feeds": []}"#);
                contents
            } else {
                return Err(Box::from("Catastrophe!"));
            }
        }
    };
    match serde_json::from_str(&config) {
        Ok(c) => Ok(c),
        Err(_) => Err(Box::from(
            "Failure to convert JSON config string to Rust struct.",
        )),
    }
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
    // Ok because we can't really recover if the config is messed up somehow
    let mut my_feeds: ConfigObj = config_to_rust().unwrap();
    my_feeds.feeds.push(Feed {
        uri: feed.to_owned(),
        last_accessed: Utc::now().to_rfc3339().to_owned(),
    });
    // Ok because this shouldn't ever panic
    rust_to_config(serde_json::to_string(&my_feeds).unwrap().as_bytes());
}

pub fn list_feeds() {
    // Ok because we can't recover if config is messed up
    let config: ConfigObj = config_to_rust().unwrap();
    // let mut uris: Vec<String> = Vec::new();
    for f in config.feeds {
        println!("{}", f.uri);
    }
}

pub fn get_uris_and_update() -> Vec<Feed> {
    let mut config = config_to_rust().unwrap();
    let mut uris: Vec<Feed> = Vec::new();
    let len = config.feeds.len();
    for i in 0..len {
        let x = config.feeds[i].to_owned();
        uris.push(x);
        config.feeds[i].last_accessed = Utc::now().to_rfc3339().to_owned();
    }
    rust_to_config(serde_json::to_string(&config).unwrap().as_bytes());
    uris
}

pub fn remove() {
    let mut config: ConfigObj = config_to_rust().unwrap();
    let mut uris: Vec<String> = Vec::new();
    let feeds_list = &config.feeds;
    for f in feeds_list {
        uris.push(f.uri.clone());
    }
    let multiselected = uris;
    let mut selections = MultiSelect::with_theme(&SimpleTheme)
        .with_prompt("Use arrow keys to move up or down. Press the space bar to select a feed. Press enter when you're done to remove all selected feeds")
        .items(&multiselected[..])
        .interact()
        .unwrap();

    // println!("{:?}", selections);
    if selections.is_empty() {
        println!("You did not select anything :(");
    } else {
        println!("Removing these feeds:");
        selections.reverse();
        for selection in selections {
            println!("  {}", multiselected[selection]);
            config.feeds.remove(selection);
        }
    }
    rust_to_config(serde_json::to_string(&config).unwrap().as_bytes())
}

pub async fn read_feed_fast(num: usize) -> Result<Vec<ProcessedFeed>, Box<dyn std::error::Error>> {
    let client = &Client::builder().build()?;

    let config_obj = config_to_rust().unwrap();
    if config_obj.feeds.len() == 0 {
        return Err(Box::from(
            "Your feeds list is empty! use `hem add` to add a feed.",
        ));
    };
    let processed = RefCell::new(Vec::<ProcessedFeed>::new());
    let fetches = futures::stream::iter(config_obj.feeds.into_iter().map(|feed| {
        let y = &processed;
        async move {
            match client.get(&feed.uri).send().await {
                Ok(resp) => match resp.text().await {
                    Ok(text) => {
                        let feed = parser::parse(text.as_bytes()).unwrap();
                        let title = feed.title.unwrap();
                        let title_owned = title.content.to_owned();

                        let entries = feed.entries.iter().enumerate();
                        let mut processed_items = Vec::<String>::new();
                        for (j, e) in entries {
                            if j < num {
                                let e_title = e.title.as_ref().unwrap();
                                processed_items.push(format!(
                                    "{} \n\t  {}\n",
                                    Style::new().italic().paint(e_title.content.clone()),
                                    e.links[0].href
                                ));
                            } else {
                                break;
                            }
                        }
                        let feed_to_add = ProcessedFeed {
                            title: title_owned,
                            items: processed_items,
                        };
                        y.borrow_mut().push(feed_to_add);
                    }
                    Err(_) => {
                        println!("ERROR reading {}", feed.uri);
                    }
                },
                Err(_) => {
                    println!("ERROR reading {}", feed.uri);
                }
            };
        }
    }))
    .buffer_unordered(20)
    .collect::<Vec<()>>();

    fetches.await;
    let x = processed.borrow();
    Ok(x.to_vec())
}

pub async fn read_feed_fast_duration() -> Result<Vec<ProcessedFeed>, Box<dyn std::error::Error>> {
    let client = &Client::builder().build()?;

    // let config_obj = config_to_rust().unwrap();
    // if config_obj.feeds.len() == 0 {
    //     return Err(Box::from(
    //         "Your feeds list is empty! use `hem add` to add a feed.",
    //     ));
    // };
    let uris = get_uris_and_update();
    if uris.len() == 0 {
        return Err(Box::from(
            "Your feeds list is empty! use `hem add` to add a feed.",
        ));
    }
    let processed = RefCell::new(Vec::<ProcessedFeed>::new());
    let fetches = futures::stream::iter(uris.into_iter().map(|config_feed| {
        let y = &processed;
        async move {
            match client.get(&config_feed.uri).send().await {
                Ok(resp) => match resp.text().await {
                    Ok(text) => {
                        let feed = match parser::parse(text.as_bytes()) {
                            Err(_) => {
                                eprintln!("Invalid RSS feed found at {}", config_feed.uri);
                                return ();
                            }
                            Ok(f) => f,
                        };
                        let last_acc_date =
                            match DateTime::parse_from_rfc3339(&config_feed.last_accessed) {
                                Err(e) => {
                                    eprintln!("Bad date formatting for {}", config_feed.uri);
                                    return ();
                                }
                                Ok(l) => l,
                            };

                        let last_accessed_parsed = DateTime::from(last_acc_date);
                        let title = feed.title.unwrap();
                        let title_owned = title.content.to_owned();

                        let entries = feed.entries.iter().enumerate();
                        let mut processed_items = Vec::<String>::new();
                        let mut entry_date;
                        for (j, e) in entries {
                            if e.updated.is_none() {
                                entry_date = e.published.unwrap();
                            } else {
                                entry_date = e.updated.unwrap();
                            }
                            let entry_duration = last_accessed_parsed - entry_date; //e.updated.unwrap();
                            if j < 5 && entry_duration.num_seconds() < 0 {
                                let e_title = e.title.as_ref().unwrap();
                                processed_items.push(format!(
                                    "{} \n\t  {}\n",
                                    Style::new().italic().paint(e_title.content.clone()),
                                    e.links[0].href
                                ));
                            } else {
                                break;
                            }
                        }
                        if processed_items.len() == 0 {
                            processed_items = vec![String::from("Nothing new here...")];
                        }
                        let feed_to_add = ProcessedFeed {
                            title: title_owned,
                            items: processed_items,
                        };
                        y.borrow_mut().push(feed_to_add);
                    }
                    Err(_) => {
                        println!("ERROR reading {}", config_feed.uri);
                    }
                },
                Err(_) => {
                    println!("ERROR reading {}", config_feed.uri);
                }
            };

            // config_feed.last_accessed = Utc::now().to_rfc3339().to_owned();
        }
    }))
    .buffer_unordered(20)
    .collect::<Vec<()>>();
    fetches.await;
    let x = processed.borrow();
    // rust_to_config(serde_json::to_string(&config_obj).unwrap().as_bytes());
    Ok(x.to_vec())
}
