use ansi_term::Style;
use chrono::offset::Utc;
use chrono::DateTime;
use feed_rs::parser;
use hemlib::{add_feed, find_config, list_feeds, rust_to_config, ConfigObj, ProcessedFeed};
use reqwest::Client;
use std::fs;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Hemingway",
    about = "An economical RSS reader for your terminal."
)]
struct Cli {
    #[structopt(subcommand)]
    sub_cmd: Option<Cmd>,
}
#[derive(StructOpt, Debug)]
enum Cmd {
    /// Adds the feed URL passed to it to your feeds list.
    Add { feed_url: String },

    /// Prints out a given number of each feed's newest entries.
    Top {
        #[structopt(default_value = "1")]
        ///The number of newest entries to display per feed.
        num_entries: usize,
    },

    /// Lists out your saved feeds.
    List,
}

async fn top<'a>(num: usize) -> Result<Vec<ProcessedFeed>, Box<dyn std::error::Error>> {
    let config_path = find_config();
    let client = Client::new();
    let mut processed: Vec<ProcessedFeed> = Vec::new();
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
                let mut bufreader = BufReader::new(configfile);
                let mut contents = String::new();
                bufreader.read_to_string(&mut contents)?;
                contents
            } else {
                return Err(Box::from("Catastrophe!"));
            }
        }
    };
    let config_obj: ConfigObj = match serde_json::from_str(&config) {
        Ok(config_obj) => config_obj,
        Err(_) => {
            return Err(Box::from(
                "Your feeds list is empty! use `hem add` to add a feed.",
            ))
        }
    };

    for i in 0..config_obj.feeds.len() {
        let resp = client
            .get(&config_obj.feeds[i].uri)
            .send()
            .await?
            .text()
            .await?;
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
                        e.id
                    ));
                }
            }

            ProcessedFeed {
                title: title_owned,
                items: processed_items,
            }
        };
        processed.push(procfeed);
    }
    rust_to_config(serde_json::to_string(&config_obj).unwrap().as_bytes());
    Ok(processed)
}

async fn process_feed<'a>() -> Result<Vec<ProcessedFeed>, Box<dyn std::error::Error>> {
    let config_path = find_config();
    let mut processed: Vec<ProcessedFeed> = Vec::new();
    let client = Client::new();
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
                let mut bufreader = BufReader::new(configfile);
                let mut contents = String::new();
                bufreader.read_to_string(&mut contents)?;
                contents
            } else {
                return Err(Box::from("Catastrophe!"));
            }
        }
    };
    let mut config_obj: ConfigObj = match serde_json::from_str(&config) {
        Ok(config_obj) => config_obj,
        Err(_) => {
            return Err(Box::from(
                "Your feeds list is empty! use `hem add` to add a feed.",
            ))
        }
    };

    for i in 0..config_obj.feeds.len() {
        let resp = client
            .get(&config_obj.feeds[i].uri)
            .send()
            .await?
            .text()
            .await?;
        let feed = parser::parse(resp.as_bytes()).unwrap();
        let last_accessed = DateTime::from(
            DateTime::parse_from_rfc3339(&config_obj.feeds[i].last_accessed).unwrap(),
        );
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
                    processed_items.push(format!("{} \n\t  {}\n", e_title.content.clone(), e.id));
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
        processed.push(procfeed);
        config_obj.feeds[i].last_accessed = Utc::now().to_rfc3339().to_owned();
    }
    rust_to_config(serde_json::to_string(&config_obj).unwrap().as_bytes());
    Ok(processed)
}

// access feeds
// if feed has been updated since last access (stored in config), then display 5 newest items
// else display "Nothing new"
// update last_access date in config
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    match args.sub_cmd {
        None => {
            let processed = process_feed().await?;
            for e in processed {
                println!("{}", e);
            }
            None
        }
        Some(i) => {
            match &i {
                Cmd::Add { feed_url } => add_feed(feed_url),
                Cmd::Top { num_entries } => {
                    let top_entries = top(*num_entries).await?;
                    for e in top_entries {
                        println!("{}", e);
                    }
                }
                Cmd::List => {
                    list_feeds();
                }
            };
            Some(i)
        }
    };

    Ok(())
}
