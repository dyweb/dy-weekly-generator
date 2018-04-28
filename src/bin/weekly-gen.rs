extern crate dy_weekly_generator;
use dy_weekly_generator::casual::Casual;
use dy_weekly_generator::error::Error;
use dy_weekly_generator::formal::Formal;
use dy_weekly_generator::github;
use dy_weekly_generator::weekly::WeeklyBuilder;

#[macro_use]
extern crate clap;
use clap::App;

extern crate json;

fn work() -> Result<(), Error> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let file = matches.value_of("file").ok_or(Error::ConfigErr)?;
    let repo = matches.value_of("repo").ok_or(Error::ConfigErr)?;
    let issue = matches.value_of("issue").ok_or(Error::ConfigErr)?;
    let key = matches.value_of("key");

    let comments = github::fetch(repo, issue, key)?;
    let mut weekly = WeeklyBuilder::new()
        .add_extractor(Box::new(Formal::new()))
        .add_extractor(Box::new(Casual::new()))
        .build();
    for body in comments.iter() {
        weekly.parse(body)
    }
    weekly.render(file)
}

fn main() {
    match work() {
        Err(Error::ConfigErr) => println!("Invalid arguments!"),
        Err(Error::RequestErr(e)) => println!("Error while sending request ({:?})", e),
        Err(Error::FetchErr) => println!("Error while fetching"),
        Err(Error::JsonParseErr) => println!("Invalid json"),
        Err(Error::IOErr(e)) => println!("Error while file operations ({:?})", e),
        Ok(_) => {}
    };
}
