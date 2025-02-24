use std::{
    collections::HashMap, io::Write, process::exit, str::FromStr, sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    }, thread, time::{Duration, Instant}
};

use clap::{arg, command, Parser, ValueEnum};
use colored::{ColoredString, Colorize};
use reqwest::{Body, Client, Method, Request, Url};
use tokio::{task, time};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    addr: String,

    #[arg(short, long, default_value_t = 25)]
    count: u32,

    #[arg(short, long, default_value_t = HttpMethod::GET)]
    method: HttpMethod,

    #[arg(short, long)]
    body: Option<String>,

    #[arg(short, long, default_value_t = 0)]
    delay: u32,

    #[arg(short, long = "expected", default_value_t = 200)]
    expected_code: u16,

    #[arg(short = 'H', long)]
    headers: Option<Vec<String>>,
}

#[tokio::main]
async fn main() {
    let prefix = "[>]".blue().bold();
    let args = Args::parse();
    let count = args.count;
    let mut tasks = vec![];
    let timer_seconds = 3;
    let client = Arc::new(Client::new());

    let request = build_request(args.clone(), prefix.clone());

    for i in (1..=timer_seconds).rev() {
        print!(
            "{prefix} Going to send {} requests to {}, in {} seconds\r",
            count.to_string().blue(),
            request.url().to_string().blue(),
            i.to_string().blue().bold()
        );
        std::io::stdout().flush().unwrap();
        thread::sleep(Duration::from_secs(1));
    }

    println!(
        "{prefix} Going to send: {} requests to: {}, {}",
        count.to_string().blue(),
        request.url().to_string().blue(),
        "has started..".blue().bold()
    );

    println!("{prefix} Waiting for requests to finish");
    let successes = Arc::new(AtomicUsize::new(0));
    let fails = Arc::new(AtomicUsize::new(0));

    let start_time = Instant::now();
    for _ in 0..count {
        let prefix = prefix.clone();
        let successes = Arc::clone(&successes);
        let fails = Arc::clone(&fails);
        let client = Arc::clone(&client);
        let request = build_request(args.clone(), prefix.clone());
        
        if args.delay != 0 {
            time::sleep(Duration::from_millis(args.delay.into())).await;
        }
   
        tasks.push(task::spawn(async move {
            match client.execute(request).await {
                Ok(response) => {
                    if response.status().as_u16() != args.expected_code {
                        fails.fetch_add(1, Ordering::Relaxed);
                        println!("{prefix} Unexpected Status: {}", response.status().to_string().red());
                    } else {
                        successes.fetch_add(1, Ordering::Relaxed);
                    }
                }
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
        "{prefix} Done ({})! Successes: {}, Fails: {}",
        format!("{} ms", start_time.elapsed().as_millis()).blue(),
        successes.load(Ordering::Relaxed).to_string().green(),
        fails.load(Ordering::Relaxed).to_string().red()
    );
}


fn build_request(args: Args, prefix: ColoredString) -> Request {
    let headers_map: HashMap<String, String> = match args.headers {
        Some(headers) => {
            headers
            .iter()
            .map(|header| {
                let parts: Vec<&str> = header.trim().split(":").collect();
                if parts.len() != 2 {
                    eprintln!("Invalid header format: '{}'. Expected 'key: value'", header);
                    std::process::exit(1);
                }
                let key = parts[0].trim().to_string();
                let value = parts[1].trim().to_string();
                (key, value)
            })
            .collect()
        },
        None => { HashMap::new() },
    };

    let method = Method::from(args.method);
    let url = Url::from_str(&add_http_if_missing(&args.addr));
    if let Err(why) = url {
        eprintln!("{prefix} Failed to build url: {why}");
        exit(1);
    }

    let mut request = Request::new(method.clone(), url.unwrap());

    for (key, value) in headers_map {
        request.headers_mut().insert(
            reqwest::header::HeaderName::from_bytes(key.as_bytes()).unwrap(),
            reqwest::header::HeaderValue::from_str(&value).unwrap(),
        );
    }

    if let Some(body) = args.body {
        if method == Method::POST || method == Method::PUT || method == Method::PATCH {
            *request.body_mut() = Some(Body::from(body));
        } else {
            eprintln!("{prefix} Body is not allowed for {} requests", method);
            exit(1);
        }
    }

    request
}

fn add_http_if_missing(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl ToString for HttpMethod {
    fn to_string(&self) -> String {
        match self {
            HttpMethod::GET => "get".into(),
            HttpMethod::POST => "post".into(),
            HttpMethod::PUT => "put".into(),
            HttpMethod::DELETE => "delete".into(),
            HttpMethod::PATCH => "patch".into(),
            HttpMethod::HEAD => "head".into(),
            HttpMethod::OPTIONS => "options".into(),
        }
    }
}

impl From<HttpMethod> for Method {
    fn from(method: HttpMethod) -> Method {
        match method {
            HttpMethod::GET => Method::GET,
            HttpMethod::POST => Method::POST,
            HttpMethod::PUT => Method::PUT,
            HttpMethod::DELETE => Method::DELETE,
            HttpMethod::PATCH => Method::PATCH,
            HttpMethod::HEAD => Method::HEAD,
            HttpMethod::OPTIONS => Method::OPTIONS,
        }
    }
}
