use std::fs::File;
use std::io;

use clap::{crate_version, App, Arg};

use dy_weekly_generator::casual::Casual;
use dy_weekly_generator::error::Error;
use dy_weekly_generator::formal::Formal;
use dy_weekly_generator::github;
use dy_weekly_generator::weekly::WeeklyBuilder;

fn work() -> Result<(), Error> {
    let matches = App::new("dy-weekly-generator")
        .version(crate_version!())
        .author("codeworm96 <codeworm96@outlook.com>")
        .about("Generates dy weekly")
        .arg(
            Arg::with_name("file")
                .short("o")
                .long("output")
                .value_name("file")
                .help("The final weekly file")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("repo")
                .short("r")
                .long("repo")
                .value_name("repo")
                .help("The github repo, like 'dyweb/weekly'")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("issue")
                .short("i")
                .long("issue")
                .value_name("issue")
                .help("The issue id")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key")
                .short("k")
                .long("key")
                .value_name("key")
                .help("The github api key")
                .takes_value(true),
        )
        .get_matches();

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
