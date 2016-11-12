#[macro_use]
extern crate clap;
use clap::App;

extern crate hyper;
use hyper::client::Client;
use hyper::header::{Headers, Accept, Authorization, UserAgent, qitem};
use hyper::mime::Mime;

use std::io::prelude::*;
use std::fs::File;

const API_ROOT: &'static str = "https://api.github.com";

#[derive(Debug)]
struct Config {
    file: String,
    repo: String,
    issue: String,
    key: Option<String>,
}

enum WeeklyErr {
    ConfigErr,
    RequestErr(hyper::error::Error),
    FetchErr,
    IOErr,
}

fn parse_args() -> Result<Config, WeeklyErr> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let file = try!(matches.value_of("file").ok_or(WeeklyErr::ConfigErr));
    let repo = try!(matches.value_of("repo").ok_or(WeeklyErr::ConfigErr));
    let issue = try!(matches.value_of("issue").ok_or(WeeklyErr::ConfigErr));
    let key = matches.value_of("key");

    Ok(Config {
        file: file.to_string(),
        repo: repo.to_string(),
        issue: issue.to_string(),
        key: key.map(|k| { k.to_string() }),
    })
}

fn fetch(config: &Config) -> Result<String, WeeklyErr> {
    let client = Client::new();
    let url = format!("{}/repos/{}/issues/{}/comments", API_ROOT, config.repo, config.issue);

    let mut headers = Headers::new();
    let accept_mime: Mime = "application/vnd.github.v3+json".parse().unwrap();
    headers.set(Accept(vec![qitem(accept_mime)]));
    headers.set(UserAgent("dy-weekly-generator/0.0.1".to_string()));
    match config.key {
        Some(ref k) => headers.set(Authorization(format!("token {}", k))),
        None => {}
    }

    let mut res = try!(client.get(&url)
                       .headers(headers)
                       .send()
                       .map_err(|e| { WeeklyErr::RequestErr(e) }));

    if res.status != hyper::Ok {
        Err(WeeklyErr::FetchErr)
    } else {
        let mut content = String::new();
        try!(res.read_to_string(&mut content).map_err(|_| { WeeklyErr::FetchErr }));
        Ok(content)
    }
}

fn render(comments: String) -> Result<String, WeeklyErr> {
    Ok(comments)
}

fn save(config: &Config, weekly: String) -> Result<(), WeeklyErr> {
    let mut file = try!(File::create(config.file.clone()).map_err(|_| { WeeklyErr::IOErr }));
    try!(write!(file, "{}", weekly).map_err(|_| { WeeklyErr::IOErr }));
    Ok(())
}

fn work() -> Result<(), WeeklyErr> {
    let config = try!(parse_args());
    let comments = try!(fetch(&config));
    let weekly = try!(render(comments));
    try!(save(&config, weekly));
    Ok(())
}

fn main() {
    match work() {
        Err(WeeklyErr::ConfigErr) => println!("Invalid arguments!"),
        Err(WeeklyErr::RequestErr(e)) => println!("Error while sending request ({:?})", e),
        Err(WeeklyErr::FetchErr) => println!("Error while fetching"),
        Err(WeeklyErr::IOErr) => println!("Error while file operations"),
        Ok(_) => {}
    };
}
