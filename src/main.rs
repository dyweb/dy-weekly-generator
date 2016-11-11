#[macro_use]
extern crate clap;
use clap::App;

use std::io::prelude::*;
use std::fs::File;

#[derive(Debug)]
struct Config {
    file: String,
    repo: String,
    issue: String,
    key: String,
}

enum WeeklyErr {
    ConfigErr,
    IOErr,
}

fn parse_args() -> Result<Config, WeeklyErr> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let file = try!(matches.value_of("file").ok_or(WeeklyErr::ConfigErr));
    let repo = try!(matches.value_of("repo").ok_or(WeeklyErr::ConfigErr));
    let issue = try!(matches.value_of("issue").ok_or(WeeklyErr::ConfigErr));
    let key = try!(matches.value_of("key").ok_or(WeeklyErr::ConfigErr));

    Ok(Config {
        file: file.to_string(),
        repo: repo.to_string(),
        issue: issue.to_string(),
        key: key.to_string(),
    })
}

fn fetch(config: &Config) -> Result<String, WeeklyErr> {
    Ok(config.key.clone())
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
        Err(WeeklyErr::IOErr) => println!("Error while file operations"),
        Ok(_) => {}
    };
}
