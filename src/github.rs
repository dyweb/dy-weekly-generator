use json;
use regex::Regex;
use reqwest;
use reqwest::header::{HeaderMap, ACCEPT, AUTHORIZATION, LINK, USER_AGENT};
use reqwest::Client;

use std::io::Read;
use std::mem;

use error::Error;

const API_ROOT: &'static str = "https://api.github.com";

pub struct Comments<'a> {
    client: Client,
    key: Option<&'a str>,
    comments: Box<Iterator<Item = String>>,
    next: Option<String>,
}

impl<'a> Iterator for Comments<'a> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        loop {
            if let Some(s) = self.comments.next() {
                return Some(s);
            } else {
                if let Some(url) = mem::replace(&mut self.next, None) {
                    if let Ok(page) = fetch_page(&self.client, &url, self.key) {
                        self.comments = page.comments;
                        self.next = page.next;
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            }
        }
    }
}

struct Page {
    comments: Box<Iterator<Item = String>>,
    next: Option<String>,
}

fn next_link(headers: &HeaderMap) -> Option<&str> {
    let link = headers.get(LINK)?.to_str().ok()?;
    let re = Regex::new(r#"<([^>]*)>; rel="next""#).unwrap();
    Some(re.captures(link)?.get(1)?.as_str())
}

fn fetch_page(client: &Client, url: &str, key: Option<&str>) -> Result<Page, Error> {
    let mut headers = HeaderMap::new();
    headers.insert(ACCEPT, "application/vnd.github.v3+json".parse().unwrap());
    headers.insert(USER_AGENT, "dy-weekly-generator/0.2.0".parse().unwrap());
    match key {
        Some(k) => {
            headers.insert(AUTHORIZATION, format!("token {}", k).parse().unwrap());
        }
        None => {}
    }

    let mut res = client.get(url).headers(headers).send()?;

    if res.status() != reqwest::StatusCode::OK {
        Err(Error::FetchErr(res))
    } else {
        let mut content = String::new();
        res.read_to_string(&mut content)?;
        let content = json::parse(&content)?;
        let comments = match content {
            json::JsonValue::Array(cs) => {
                Ok(Box::new(cs.into_iter().flat_map(
                    |mut c| match c["body"].take() {
                        json::JsonValue::String(s) => Some(s),
                        _ => None,
                    },
                )))
            }
            _ => Err(Error::JsonParseErr),
        }?;
        Ok(Page {
            comments: comments,
            next: next_link(res.headers()).map(|s| s.to_owned()),
        })
    }
}

pub fn fetch<'a>(repo: &str, issue: &str, key: Option<&'a str>) -> Result<Comments<'a>, Error> {
    let client = Client::new();
    let url = format!("{}/repos/{}/issues/{}/comments", API_ROOT, repo, issue);
    let page = fetch_page(&client, &url, key)?;
    Ok(Comments {
        client: client,
        key: key,
        comments: page.comments,
        next: page.next,
    })
}
