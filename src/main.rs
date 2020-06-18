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
#[structopt(name = "Hemingway", about = "a small RSS reader")]
struct Cli {
    #[structopt(subcommand)]
    add_cmd: Option<Cmd>,
}
#[derive(StructOpt, Debug)]
enum Cmd {
    /// Adds the feed URL passed to it to your feeds list.
    Add { feed_url: String },
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

fn add_feed(feed: &str) {
    let homedir: std::path::PathBuf = dirs::home_dir().expect("no home dir");
    let path_to_config: &Path = &Path::new(&homedir).join(".hemrc");
    let config = fs::read_to_string(path_to_config).expect("reading config failed");
    let mut my_feeds: ConfigObj = serde_json::from_str(&config).unwrap();
    my_feeds.feeds.push(Feed {
        uri: feed.to_owned(),
        last_accessed: "hello".to_owned(),
    });
    let mut file = match File::create(path_to_config) {
        Err(why) => panic!("config file access failed: {}", why),
        Ok(file) => file,
    };

    match file.write_all(serde_json::to_string(&my_feeds).unwrap().as_bytes()) {
        Err(why) => panic!("config file writing failed: {}", why),
        Ok(_) => println!("feed added"),
    };
}

async fn process_feed<'a>() -> Result<Vec<ProcessedFeed>, Box<dyn std::error::Error>> {
    let mut processed: Vec<ProcessedFeed> = Vec::new(); //.into_iter().enumerate().map(|(i, e)| {println!("hello"); (i, e)}).collect();
                                                        // let configdir: std::path::PathBuf = match config_dir() {
                                                        //     None => return Err(Box::from("no config dir")),
                                                        //     Some(configdir) => {
                                                        //         println!("{:?}", configdir);
                                                        //         configdir
                                                        //     }
                                                        // };
    let homedir: std::path::PathBuf = match dirs::home_dir() {
        None => return Err(Box::from("You need a home directory...")),
        Some(homedir) => homedir,
    };
    let path_to_config: &Path = &Path::new(&homedir).join(".hemrc");
    let config = match fs::read_to_string(path_to_config) {
        Ok(config) => config,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                eprintln!("Didn't find a .hemrc, creating it now...");
                // create the file and populate it with  an empty array
                let mut configfile = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .create_new(true)
                    .open(path_to_config)?;
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
    let config_obj: ConfigObj = serde_json::from_str(&config)?;
    // let mut feed: model::Feed;
    // let resp = reqwest::get(&args.feed).await?.text().await?;
    if config_obj.feeds.len() == 0 {
        return Err(Box::from(
            "Your feeds list is empty! use `hem add` to add a feed.",
        ));
    }
    for f in config_obj.feeds.iter() {
        let resp = reqwest::get(&f.uri).await?.text().await?;
        let feed = parser::parse(resp.as_bytes()).unwrap();
        let procfeed = {
            // let feedref = &feed;
            let title = feed.title.unwrap();
            let title_owned = title.content.to_owned();

            // println!("{}", feed.title.unwrap().content);
            // let y = &x.content.as_ref().unwrap();
            // println!("{:?}", x.title.as_ref().unwrap());
            let entries = feed.entries.iter().enumerate();
            let mut it = Vec::<String>::new();
            for (j, e) in entries {
                if j < 5 {
                    // println!("\t{} : {}", e.title.as_ref().unwrap().content, e.id);
                    let et = e.title.as_ref().unwrap();
                    it.push(format!("{} 🔗 {}", et.content.clone(), e.id));
                }
            }

            ProcessedFeed {
                title: title_owned,
                items: it,
            }
        };
        processed.push(procfeed);
        // println!("{:?}", processed);
    }
    Ok(processed)
}

// access feeds
// if feed has been updated since last access (stored in config), then display 5 newest items
// else display "Nothing new"
// update last_access date in config
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    // println!("{:?}", args);
    match args.add_cmd {
        None => {
            let processed = process_feed().await?;
            for e in processed {
                println!("{}", e);
            }
            // println!("{}", processed.unwrap());
            None
        }
        Some(i) => {
            match &i {
                Cmd::Add { feed_url } => add_feed(feed_url),
            };
            Some(i)
        }
    };
    // match args {
    //     // None => None,
    //     // Some(i) => {
    //     //     println!("{}", i);
    //     //     Some(i)
    //     // }
    //     Cli::Add => println!("got em"),
    // };
    // println!("{}", x.unwrap());
    // if

    Ok(())
}
