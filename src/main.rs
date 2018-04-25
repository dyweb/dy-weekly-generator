extern crate yaml_rust;

extern crate regex;
use regex::Regex;

mod weekly;
use weekly::Weekly;

#[macro_use]
extern crate clap;
use clap::App;

extern crate reqwest;
use reqwest::Client;
use reqwest::header::{Headers, Accept, Authorization, UserAgent, qitem};
use reqwest::mime::Mime;

extern crate json;

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

fn parse_args() -> Result<Config, weekly::Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let file = matches.value_of("file").ok_or(weekly::Error::ConfigErr)?;
    let repo = matches.value_of("repo").ok_or(weekly::Error::ConfigErr)?;
    let issue = matches.value_of("issue").ok_or(weekly::Error::ConfigErr)?;
    let key = matches.value_of("key");

    Ok(Config {
        file: file.to_string(),
        repo: repo.to_string(),
        issue: issue.to_string(),
        key: key.map(|k| { k.to_string() }),
    })
}

fn fetch(config: &Config) -> Result<String, weekly::Error> {
    let client = Client::new();
    let url = format!("{}/repos/{}/issues/{}/comments", API_ROOT, config.repo, config.issue);

    let mut headers = Headers::new();
    let accept_mime: Mime = "application/vnd.github.v3+json".parse().unwrap();
    headers.set(Accept(vec![qitem(accept_mime)]));
    headers.set(UserAgent::new("dy-weekly-generator/0.2.0".to_string()));
    match config.key {
        Some(ref k) => headers.set(Authorization(format!("token {}", k))),
        None => {}
    }

    let mut res = client.get(&url)
                    .headers(headers)
                    .send()
                    .map_err(|e| { weekly::Error::RequestErr(e) })?;

    if res.status() != reqwest::StatusCode::Ok {
        Err(weekly::Error::FetchErr)
    } else {
        let mut content = String::new();
        res.read_to_string(&mut content).map_err(|_| { weekly::Error::FetchErr })?;
        Ok(content)
    }
}

fn parse_comment(weekly: &mut Weekly, comment: &str) {
    println!("{}", comment); // dump comments for manual editing
    let begin = Regex::new(r"```[:space:]*(yaml|yml)").unwrap();
    let end = Regex::new(r"```").unwrap();
    let mut entry = String::new();
    let mut in_yaml = false;
    for line in comment.lines() {
        if begin.is_match(line) {
            entry = String::new();
            in_yaml = true;
        } else if end.is_match(line) {
            weekly.parse(&entry);
            in_yaml = false;
        } else if in_yaml {
            entry.push_str(line);
            entry.push_str("\n");
        }
    }
}

fn parse(comments: String) -> Result<Weekly, weekly::Error> {
    let comment_list = json::parse(&comments).map_err(|_| { weekly::Error::JsonParseErr })?;
    let mut weekly = Weekly::new();
    match comment_list {
        json::JsonValue::Array(cs) => {
            for c in &cs {
                if let Some(body) = c["body"].as_str() {
                    parse_comment(&mut weekly, body); 
                }
            }
            Ok(weekly)
        }
        _ => Err(weekly::Error::JsonParseErr),
    }
}

fn render(config: &Config, weekly: Weekly) -> Result<(), weekly::Error> {
    let file = File::create(config.file.clone()).map_err(|_| { weekly::Error::IOErr })?;
    weekly.render(file)
}

fn work() -> Result<(), weekly::Error> {
    let config = parse_args()?;
    let comments = fetch(&config)?;
    let weekly = parse(comments)?;
    render(&config, weekly)?;
    Ok(())
}

fn main() {
    match work() {
        Err(weekly::Error::ConfigErr) => println!("Invalid arguments!"),
        Err(weekly::Error::RequestErr(e)) => println!("Error while sending request ({:?})", e),
        Err(weekly::Error::FetchErr) => println!("Error while fetching"),
        Err(weekly::Error::JsonParseErr) => println!("Invalid json"),
        Err(weekly::Error::IOErr) => println!("Error while file operations"),
        Ok(_) => {}
    };
}
