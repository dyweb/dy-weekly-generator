use json;
use reqwest;
use reqwest::header::{qitem, Accept, Authorization, Headers, UserAgent};
use reqwest::mime::Mime;
use reqwest::Client;

use std::io::Read;

use error::Error;

const API_ROOT: &'static str = "https://api.github.com";

pub fn fetch<'a>(
    repo: &str,
    issue: &str,
    key: Option<&str>,
) -> Result<impl Iterator<Item = String>, Error> {
    let client = Client::new();
    let url = format!("{}/repos/{}/issues/{}/comments", API_ROOT, repo, issue);

    let mut headers = Headers::new();
    let accept_mime: Mime = "application/vnd.github.v3+json".parse().unwrap();
    headers.set(Accept(vec![qitem(accept_mime)]));
    headers.set(UserAgent::new("dy-weekly-generator/0.2.0".to_string()));
    match key {
        Some(k) => headers.set(Authorization(format!("token {}", k))),
        None => {}
    }

    let mut res = client.get(&url).headers(headers).send()?;

    if res.status() != reqwest::StatusCode::Ok {
        Err(Error::FetchErr)
    } else {
        let mut content = String::new();
        res.read_to_string(&mut content)?;
        let comments = json::parse(&content)?;
        match comments {
            json::JsonValue::Array(cs) => {
                Ok(cs.into_iter().flat_map(|mut c| match c["body"].take() {
                    json::JsonValue::String(s) => Some(s),
                    _ => None,
                }))
            }
            _ => Err(Error::JsonParseErr),
        }
    }
}
