use std::collections::HashMap;
use std::mem;
use std::fs::File;
use std::io::prelude::*;

use yaml_rust::YamlLoader;
use reqwest;

pub enum Error {
    ConfigErr,
    RequestErr(reqwest::Error),
    FetchErr,
    JsonParseErr,
    IOErr,
}

enum EntryType { Draft, Topic }

pub struct Entry {
    name: String,
    kind: EntryType,
    link: Option<String>,
    description: Option<String>,
    quote: Option<String>,
    cc: Vec<String>,
    // TODO: tag? keyword?
}

pub struct Weekly {
    entries: HashMap<String, Entry>,
}

impl Entry {
    fn parse(yaml: &str) -> Option<Entry> {
        YamlLoader::load_from_str(yaml).ok().and_then(|docs| {
            docs.iter().next().and_then(|doc| {
                let name = doc["name"].as_str().map(|s| { s.to_string() });
                let kind = match doc["type"].as_str() {
                    Some("draft") => Some(EntryType::Draft),
                    Some("topic") => Some(EntryType::Topic),
                    Some(_) => None,
                    None => Some(EntryType::Draft),
                };
                let link = doc["link"].as_str().map(|s| { s.to_string() });
                let description = doc["description"].as_str().map(|s| { s.to_string() });
                let quote = doc["quote"].as_str().map(|s| { s.to_string() });

                let mut cc = Vec::new();
                if let Some(raw_cc) = doc["cc"].as_vec() {
                    for person in raw_cc {
                        match person.as_str() {
                            Some(c) => cc.push(c.to_string()),
                            None => {}
                        }
                    }
                } else {
                    doc["cc"].as_str()
                        .map(|s| { format!("[{}]", s) })
                        .and_then(|s| {
                            YamlLoader::load_from_str(&s).ok().and_then(|ds| {
                                ds.iter().next()
                                    .and_then(|d| { d.as_vec() })
                                    .map(|v| {
                                    for person in v {
                                        match person.as_str() {
                                            Some(c) => cc.extend(c.split(' ').map(|s| s.to_string())),
                                            None => {}
                                        }
                                    }
                                })
                            })
                        });
                }

                match (name, kind) {
                    (Some(name), Some(kind)) => Some(Entry {
                        name: name,
                        kind: kind,
                        link: link,
                        description: description,
                        quote: quote,
                        cc: cc,
                    }),
                    _ => None,
                }
            })
        })
    }

    fn field_append(a: &mut Option<String>, b: &mut Option<String>) {
        match mem::replace(b, None) {
            Some(s2) => {
                if a.is_some() {
                    a.as_mut().map(|s1| { s1.push_str(&s2) });
                } else {
                    mem::replace(a, Some(s2));
                }
            }
            None => {}
        }
    }

    fn merge(&mut self, mut other: Entry) {
        assert_eq!(self.name, other.name);
        self.kind = other.kind;
        Self::field_append(&mut self.link, &mut other.link);
        Self::field_append(&mut self.description, &mut other.description);
        Self::field_append(&mut self.quote, &mut other.quote);
        self.cc.append(&mut other.cc);
    }

    fn render(&self, file: &mut File) -> Result<(), Error> {
        write!(file, "- ").map_err(|_| { Error::IOErr })?;
        match self.link.as_ref() {
            Some(link) => write!(file, "[{}]({})", self.name, link).map_err(|_| { Error::IOErr })?,
            None => write!(file, "{}", self.name).map_err(|_| { Error::IOErr })?,
        }
        match self.description.as_ref() {
            Some(desc) => write!(file, ", {}\n", desc).map_err(|_| { Error::IOErr })?,
            None => write!(file, "\n").map_err(|_| { Error::IOErr })?,
        }
        match self.quote.as_ref() {
            Some(quote) => {
                for line in quote.lines() {
                    write!(file, " > {}\n", line).map_err(|_| { Error::IOErr })?;
                }
            }
            None => {}
        }
        if self.cc.len() > 0 {
            let cc_list: Vec<_> = self.cc.iter().map(|person| { format!("[@{}](https://github.com/{})", person, person) }).collect();
            write!(file, "{}\n", cc_list.join(", ")).map_err(|_| { Error::IOErr })?;
        }
        Ok(())
    }
}

impl Weekly {
    pub fn new() -> Weekly {
        Weekly {
            entries: HashMap::new(),
        }
    }

    pub fn parse(&mut self, yaml: &str) {
        let entry = Entry::parse(yaml);
        match entry {
            Some(e) => {
                if let Some(ent) = self.entries.get_mut(&e.name) {
                    ent.merge(e);
                    return;
                }
                self.entries.insert(e.name.clone(), e);
            }
            None => {},
        }
    }

    pub fn render(&self, mut file: File) -> Result<(), Error> {
        let header = r#"---
layout: post
title: Weekly
category: Weekly
author: 东岳

---

"#;
        write!(file, "{}", header).map_err(|_| { Error::IOErr })?;
        for entry in self.entries.values() {
            entry.render(&mut file)?;
        }
        Ok(())
    }
}
