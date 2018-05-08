use json;
use reqwest;
use reqwest::header::{qitem, Accept, Authorization, Headers, Link, RelationType, UserAgent};
use reqwest::mime::Mime;
use reqwest::Client;

use std::io::Read;
use std::mem;
use std::vec;

use error::Error;

const API_ROOT: &'static str = "https://api.github.com";

pub struct Comments<'a> {
    client: Client,
    key: Option<&'a str>,
    comments: vec::IntoIter<String>,
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
    comments: vec::IntoIter<String>,
    next: Option<String>,
}

fn next_link(headers: &Headers) -> Option<&str> {
    let link: &Link = headers.get()?;
    for v in link.values() {
        if let Some(&[RelationType::Next]) = v.rel() {
            return Some(v.link());
        }
    }
    None
}

fn fetch_page(client: &Client, url: &str, key: Option<&str>) -> Result<Page, Error> {
    let mut headers = Headers::new();
    let accept_mime: Mime = "application/vnd.github.v3+json".parse().unwrap();
    headers.set(Accept(vec![qitem(accept_mime)]));
    headers.set(UserAgent::new("dy-weekly-generator/0.2.0".to_string()));
    match key {
        Some(k) => headers.set(Authorization(format!("token {}", k))),
        None => {}
    }

    let mut res = client.get(url).headers(headers).send()?;

    if res.status() != reqwest::StatusCode::Ok {
        Err(Error::FetchErr(res))
    } else {
        let mut content = String::new();
        res.read_to_string(&mut content)?;
        let content = json::parse(&content)?;
        let comments = match content {
            json::JsonValue::Array(cs) => Ok(cs.into_iter()
                .flat_map(|mut c| match c["body"].take() {
                    json::JsonValue::String(s) => Some(s),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .into_iter()),
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
