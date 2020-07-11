use ansi_term::Style;
use chrono::offset::Utc;
use chrono::DateTime;
use feed_rs::parser;
use hemlib::ProcessedFeed;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;
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
    Add {
        feed_url: String,
    },

    Top {
        #[structopt(default_value = "1")]
        post_num: usize,
    },
    List,
}
#[derive(Debug, Serialize, Deserialize)]
struct Feed {
    uri: String,
    last_accessed: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ConfigObj {
    feeds: Vec<Feed>,
}

fn find_config() -> std::path::PathBuf {
    let homedir: std::path::PathBuf = dirs::home_dir().expect("no home dir");
    //let path_to_config: &Path =
    Path::new(&homedir).join(".hemrc")
}

fn config_to_rust() -> ConfigObj {
    let config = fs::read_to_string(&find_config()).expect("reading config failed");
    serde_json::from_str(&config).unwrap()
}

fn rust_to_config(content: &[u8]) {
    let mut file = match File::create(find_config()) {
        Err(why) => panic!("config file access failed: {}", why),
        Ok(file) => file,
    };
    file.write_all(content)
        .expect("Writing to .hemrc failed :(");
}

fn add_feed(feed: &str) {
    let mut my_feeds: ConfigObj = config_to_rust();
    my_feeds.feeds.push(Feed {
        uri: feed.to_owned(),
        last_accessed: Utc::now().to_rfc3339().to_owned(),
    });
    rust_to_config(serde_json::to_string(&my_feeds).unwrap().as_bytes());
}

fn list_feeds() {
    let my_feeds: ConfigObj = config_to_rust();
    for f in my_feeds.feeds {
        println!("{}", f.uri);
    }
}

async fn top<'a>(num: usize) -> Result<Vec<ProcessedFeed>, Box<dyn std::error::Error>> {
    let config_path = find_config();
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
        let resp = reqwest::get(&config_obj.feeds[i].uri).await?.text().await?;
        let feed = parser::parse(resp.as_bytes()).unwrap();
        let procfeed = {
            let title = feed.title.unwrap();
            let title_owned = title.content.to_owned();

            let entries = feed.entries.iter().enumerate();
            let mut it = Vec::<String>::new();
            for (j, e) in entries {
                if j < num {
                    let e_title = e.title.as_ref().unwrap();
                    it.push(format!(
                        "{} \n\t  {}\n",
                        Style::new().italic().paint(e_title.content.clone()),
                        e.id
                    ));
                }
            }

            ProcessedFeed {
                title: title_owned,
                items: it,
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
        let resp = reqwest::get(&config_obj.feeds[i].uri).await?.text().await?;
        let feed = parser::parse(resp.as_bytes()).unwrap();
        let last_accessed = DateTime::from(
            DateTime::parse_from_rfc3339(&config_obj.feeds[i].last_accessed).unwrap(),
        );
        let feed_updates = feed.updated;
        if feed_updates.is_none() {
            eprintln!(
                "{}: no update data available for this feed",
                feed.title.unwrap().content
            );
            config_obj.feeds[i].last_accessed = Utc::now().to_rfc3339().to_owned();
            continue;
        }
        let duration = last_accessed - feed_updates.unwrap();
        if duration.num_seconds() > 0 {
            println!("{}: Nothing new here...", feed.title.unwrap().content);
            config_obj.feeds[i].last_accessed = Utc::now().to_rfc3339().to_owned();
            continue;
        }
        let procfeed = {
            let title = feed.title.unwrap();
            let title_owned = title.content.to_owned();

            let entries = feed.entries.iter().enumerate();
            let mut it = Vec::<String>::new();
            for (j, e) in entries {
                let entry_duration = last_accessed - e.updated.unwrap();
                if j < 10 && entry_duration.num_seconds() < 0 {
                    let e_title = e.title.as_ref().unwrap();
                    it.push(format!("{} \n\t  {}\n", e_title.content.clone(), e.id));
                }
            }

            ProcessedFeed {
                title: title_owned,
                items: it,
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
                Cmd::Top { post_num } => {
                    let top_entries = top(*post_num).await?;
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
