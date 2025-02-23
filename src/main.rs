use std::{
    io::Write, sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    }, thread, time::Duration
};

use clap::{arg, command, Parser};
use colored::Colorize;
use tokio::task;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    addr: String,

    #[arg(short, long, default_value_t = 25)]
    count: u32,

    #[arg(long, default_value_t = true)]
    https: bool,
}

#[tokio::main]
async fn main() {
    let prefix = "[>]".blue().bold();
    let args = Args::parse();
    let count = args.count;
    let addr = add_https_if_missing(&args.addr, args.https);
    let mut tasks = vec![];
    let timer_seconds = 3;

    for i in (1..=timer_seconds).rev() {
        print!("{prefix} Going to send {} requests to {}, in {} seconds\r", 
               count.to_string().blue(), addr.blue(), i.to_string().purple().bold());
        std::io::stdout().flush().unwrap();
        thread::sleep(Duration::from_secs(1));
    }

    println!("{prefix} Going to send: {} requests to: {}, {}", count.to_string().blue(), addr.blue(), "now starting..".purple().bold());
    thread::sleep(Duration::from_millis(850));

    println!("{prefix} Waiting for requests to finish");
    let successes = Arc::new(AtomicUsize::new(0));
    let fails = Arc::new(AtomicUsize::new(0));

    for _ in 0..count {
        let url = addr.clone();
        let prefix = prefix.clone();
        let successes = Arc::clone(&successes);
        let fails = Arc::clone(&fails);

        tasks.push(task::spawn(async move {
            match reqwest::get(url).await {
                Ok(_) => { successes.fetch_add(1, Ordering::Relaxed); }
                Err(why) => {
                    fails.fetch_add(1, Ordering::Relaxed);
                    println!("{prefix} Request failed: {}", why.to_string().red());
                }
            }
        }));
    }

    for task in tasks {
        task.await.unwrap();
    }

    println!(
        "{prefix} Done! Successes: {}, Fails: {}",
        successes.load(Ordering::Relaxed).to_string().green(),
        fails.load(Ordering::Relaxed).to_string().red()
    );
}

fn add_https_if_missing(url: &str, https: bool) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        if https {
            format!("https://{}", url)
        } else {
            format!("http://{}", url)
        }
        
    }
}