use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use clap::{arg, command, Parser};
use colored::Colorize;
use tokio::task;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    addr: String,

    #[arg(short, long, default_value_t = 1)]
    count: u32,
}

#[tokio::main]
async fn main() {
    let prefix = "[>]".blue().bold();
    let args = Args::parse();
    let count = args.count;
    let addr = add_https_if_missing(&args.addr);
    let mut tasks = vec![];

    println!("{prefix} Going to send: {} requests to: {}, in 3 seconds", count.to_string().blue(), addr.blue());
    thread::sleep(Duration::from_secs(3));

    let successes = Arc::new(AtomicUsize::new(0));
    let fails = Arc::new(AtomicUsize::new(0));

    for i in 0..count {
        let url = addr.clone();
        let prefix = prefix.clone();
        
        // Cloning the Arc references for the task
        let successes = Arc::clone(&successes);
        let fails = Arc::clone(&fails);

        tasks.push(task::spawn(async move {
            match reqwest::get(url).await {
                Ok(response) => {
                    println!("{prefix} ({i}) Response status: {}", response.status());
                    successes.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    eprintln!("{prefix} Request failed: {}", e.to_string().red());
                    fails.fetch_add(1, Ordering::Relaxed);
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


fn add_https_if_missing(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}