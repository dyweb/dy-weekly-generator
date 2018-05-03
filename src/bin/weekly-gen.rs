extern crate dy_weekly_generator;
use dy_weekly_generator::casual::Casual;
use dy_weekly_generator::error::Error;
use dy_weekly_generator::formal::Formal;
use dy_weekly_generator::github;
use dy_weekly_generator::weekly::WeeklyBuilder;

#[macro_use]
extern crate clap;
use clap::App;

use std::fs::File;
use std::io;

fn work() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let file = matches.value_of("file");
    let repo = matches.value_of("repo").ok_or(Error::ConfigErr)?;
    let issue = matches.value_of("issue").ok_or(Error::ConfigErr)?;
    let key = matches.value_of("key");

    let mut weekly = WeeklyBuilder::new()
        .add_extractor(Box::new(Formal::new()))
        .add_extractor(Box::new(Casual::new()))
        .build();
    for body in github::fetch(repo, issue, key)? {
        weekly.parse(&body);
    }
    if let Some(filename) = file {
        let mut f = File::create(filename)?;
        weekly.render(&mut f)
    } else {
        weekly.render(&mut io::stdout())
    }
}

fn main() {
    match work() {
        Err(Error::ConfigErr) => println!("Invalid arguments!"),
        Err(Error::RequestErr(e)) => println!("Error while sending request ({:?})", e),
        Err(Error::FetchErr(r)) => println!("Error while fetching ({:#?})", r),
        Err(Error::JsonParseErr) => println!("Invalid json"),
        Err(Error::IOErr(e)) => println!("Error while file operations ({:?})", e),
        Ok(_) => {}
    };
}
