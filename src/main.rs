use std::env;

#[derive(Debug)]
struct Config {
    output: String,
    repo: String,
    issue: String,
    key: String,
}

enum WeeklyErr {
    ConfigErr,
}

fn parse_args() -> Result<Config, WeeklyErr> {
    let mut args = env::args();
    args.next();
    let output = try!(args.next().ok_or(WeeklyErr::ConfigErr));
    let repo = try!(args.next().ok_or(WeeklyErr::ConfigErr));
    let issue = try!(args.next().ok_or(WeeklyErr::ConfigErr));
    let key = try!(args.next().ok_or(WeeklyErr::ConfigErr));
    match args.next() {
        Some(_) => Err(WeeklyErr::ConfigErr),
        None => Ok(Config {
            output: output,
            repo: repo,
            issue: issue,
            key: key
        }),
    }
}

fn main() {
    let config = match parse_args() {
        Ok(c) => c,
        Err(_) => {
            println!("Usage: dy-weekly-generator <output> <repo> <issue> <key>");
            return;
        }
    };
    println!("{:?}", config);
}
