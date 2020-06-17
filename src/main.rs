use structopt::StructOpt;
// use rss::Channel;
use feed_rs::parser;
// use serde_json::Value;
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::Write;

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
    last_updated: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FeedList {
    feeds: Vec<Feed>,
}

fn add_feed(feed: &str) {
    let config = fs::read_to_string("./hem.json").expect("reading config failed");
    let mut my_feeds: FeedList = serde_json::from_str(&config).unwrap();
    my_feeds.feeds.push(Feed {
        uri: feed.to_owned(),
        last_updated: "hello".to_owned(),
    });
    let mut file = match File::create("hem.json") {
        Err(why) => panic!("config file access failed: {}", why),
        Ok(file) => file,
    };

    match file.write_all(serde_json::to_string(&my_feeds).unwrap().as_bytes()) {
        Err(why) => panic!("config file writing failed: {}", why),
        Ok(_) => println!("feed added"),
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::from_args();
    // println!("{:?}", args);
    match args.add_cmd {
        None => {
            let config = fs::read_to_string("./hem.json").expect("reading config failed");
            let my_feeds: FeedList = serde_json::from_str(&config)?;
            // let resp = reqwest::get(&args.feed).await?.text().await?;
            for f in my_feeds.feeds {
                let resp = reqwest::get(&f.uri).await?.text().await?;
                let feed = parser::parse(resp.as_bytes()).unwrap();
                println!("{}", feed.title.unwrap().content);
                // let y = &x.content.as_ref().unwrap();
                // println!("{:?}", x.title.as_ref().unwrap());
                let entries = &feed.entries;
                for (i, e) in entries.iter().enumerate() {
                    if i < 5 {
                        println!("\t{}", e.title.as_ref().unwrap().content);
                    }
                }
            }
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
