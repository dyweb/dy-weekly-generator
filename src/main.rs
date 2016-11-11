#[macro_use]
extern crate clap;
use clap::App;

#[derive(Debug)]
struct Config {
    file: String,
    repo: String,
    issue: String,
    key: String,
}

enum WeeklyErr {
    ConfigErr,
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

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(_) => {
            println!("Invalid arguments!");
            return;
        }
    };
    println!("{:?}", config);
}
