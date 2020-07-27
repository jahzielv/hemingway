use hemlib::{add_feed, list_feeds, read_feed_fast, read_feed_fast_duration, remove};
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

    /// Remove a saved feed
    Remove,
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
            let processed = read_feed_fast_duration().await?; //hem().await?;
            for e in processed {
                println!("{}", e);
            }
            None
        }
        Some(i) => {
            match &i {
                Cmd::Add { feed_url } => add_feed(feed_url),
                Cmd::Top { num_entries } => {
                    // let top_entries = top(*num_entries).await?;
                    // for e in top_entries {
                    //     println!("{}", e);
                    // }
                    let top_entries = read_feed_fast(*num_entries).await?;
                    for e in top_entries {
                        println!("{}", e);
                    }
                }
                Cmd::List => {
                    list_feeds();
                }
                Cmd::Remove => {
                    remove();
                }
            };
            Some(i)
        }
    };

    Ok(())
}
