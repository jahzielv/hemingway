use ansi_term::Style;
use chrono::offset::Utc;
use chrono::DateTime;
use dialoguer::{theme::SimpleTheme, MultiSelect};
use feed_rs::parser;
use futures::stream::StreamExt;
use indicatif::ProgressBar;
use itertools::Itertools;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::convert::TryFrom;
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
    Ok(serde_json::from_str(&config).unwrap())
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
    let mut my_feeds: ConfigObj = config_to_rust().unwrap();
    my_feeds.feeds.push(Feed {
        uri: feed.to_owned(),
        last_accessed: Utc::now().to_rfc3339().to_owned(),
    });
    rust_to_config(serde_json::to_string(&my_feeds).unwrap().as_bytes());
}

pub fn list_feeds() {
    let config: ConfigObj = config_to_rust().unwrap();
    // let mut uris: Vec<String> = Vec::new();
    for f in config.feeds {
        println!("{}", f.uri);
    }
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

async fn read_feed(
    url: &String,
    client: &Client,
    num: usize,
) -> Result<ProcessedFeed, Box<dyn std::error::Error>> {
    let resp = client.get(url).send().await?.text().await?;
    let feed = parser::parse(resp.as_bytes()).unwrap();
    let procfeed = {
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

        ProcessedFeed {
            title: title_owned,
            items: processed_items,
        }
    };
    Ok(procfeed)
}

async fn read_feed_duration(
    url: &String,
    client: &Client,
    last_accessed: &String,
) -> Result<ProcessedFeed, Box<dyn std::error::Error>> {
    let resp = client.get(url).send().await?.text().await?;
    let feed = parser::parse(resp.as_bytes()).unwrap();
    let last_accessed = DateTime::from(DateTime::parse_from_rfc3339(last_accessed).unwrap());
    let procfeed = {
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
            let entry_duration = last_accessed - entry_date; //e.updated.unwrap();
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
            processed_items.push(format!("Nothing new here..."));
        }

        ProcessedFeed {
            title: title_owned,
            items: processed_items,
        }
    };
    Ok(procfeed)
}

pub async fn top<'a>(num: usize) -> Result<Vec<ProcessedFeed>, Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut processed: Vec<ProcessedFeed> = Vec::new();

    let config_obj = config_to_rust().unwrap();
    if config_obj.feeds.len() == 0 {
        return Err(Box::from(
            "Your feeds list is empty! use `hem add` to add a feed.",
        ));
    };
    let bar = ProgressBar::new(u64::try_from(config_obj.feeds.len()).unwrap());
    // bar.println("Loading feeds...");

    for i in 0..config_obj.feeds.len() {
        let proc_feed = read_feed(&config_obj.feeds[i].uri, &client, num).await?;
        processed.push(proc_feed);
        bar.inc(1);
    }
    rust_to_config(serde_json::to_string(&config_obj).unwrap().as_bytes());
    bar.finish_and_clear();
    Ok(processed)
}

pub async fn hem<'a>() -> Result<Vec<ProcessedFeed>, Box<dyn std::error::Error>> {
    let mut processed: Vec<ProcessedFeed> = Vec::new();
    let client = Client::new();

    let mut config_obj = config_to_rust().unwrap();
    if config_obj.feeds.len() == 0 {
        return Err(Box::from(
            "Your feeds list is empty! use `hem add` to add a feed.",
        ));
    };

    let bar = ProgressBar::new(u64::try_from(config_obj.feeds.len()).unwrap());
    for i in 0..config_obj.feeds.len() {
        let proc_feed = read_feed_duration(
            &config_obj.feeds[i].uri,
            &client,
            &config_obj.feeds[i].last_accessed,
        )
        .await?;
        processed.push(proc_feed);
        config_obj.feeds[i].last_accessed = Utc::now().to_rfc3339().to_owned();
        bar.inc(1);
    }
    rust_to_config(serde_json::to_string(&config_obj).unwrap().as_bytes());
    bar.finish_and_clear();
    Ok(processed)
}
