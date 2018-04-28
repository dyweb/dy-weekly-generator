use std::collections::HashMap;
use std::io;
use std::mem;

use regex::Regex;
use yaml_rust::YamlLoader;

use error::Error;
use weekly::Extractor;

enum EntryType {
    Draft,
    Topic,
}

struct Entry {
    name: String,
    kind: EntryType,
    link: Option<String>,
    description: Option<String>,
    quote: Option<String>,
    cc: Vec<String>,
    // TODO: tag? keyword?
}

impl Entry {
    fn parse(yaml: &str) -> Option<Entry> {
        YamlLoader::load_from_str(yaml).ok().and_then(|docs| {
            docs.iter().next().and_then(|doc| {
                let name = doc["name"].as_str().map(|s| s.to_string());
                let kind = match doc["type"].as_str() {
                    Some("draft") => Some(EntryType::Draft),
                    Some("topic") => Some(EntryType::Topic),
                    Some(_) => None,
                    None => Some(EntryType::Draft),
                };
                let link = doc["link"].as_str().map(|s| s.to_string());
                let description = doc["description"].as_str().map(|s| s.to_string());
                let quote = doc["quote"].as_str().map(|s| s.to_string());

                let mut cc = Vec::new();
                if let Some(raw_cc) = doc["cc"].as_vec() {
                    for person in raw_cc {
                        match person.as_str() {
                            Some(c) => cc.push(c.to_string()),
                            None => {}
                        }
                    }
                } else {
                    doc["cc"]
                        .as_str()
                        .map(|s| format!("[{}]", s))
                        .and_then(|s| {
                            YamlLoader::load_from_str(&s).ok().and_then(|ds| {
                                ds.iter().next().and_then(|d| d.as_vec()).map(|v| {
                                    for person in v {
                                        match person.as_str() {
                                            Some(c) => {
                                                cc.extend(c.split(' ').map(|s| s.to_string()))
                                            }
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
                    a.as_mut().map(|s1| s1.push_str(&s2));
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
        Self::field_append(&mut self.link, &mut other.link); // FIXME: not a proper way to merge link
        Self::field_append(&mut self.description, &mut other.description);
        Self::field_append(&mut self.quote, &mut other.quote);
        self.cc.append(&mut other.cc);
    }

    fn render(&self, out: &mut io::Write) -> Result<(), Error> {
        write!(out, "- ")?;
        match self.link.as_ref() {
            Some(link) => write!(out, "[{}]({})", self.name, link)?,
            None => write!(out, "{}", self.name)?,
        }
        match self.description.as_ref() {
            Some(desc) => write!(out, ", {}\n", desc)?,
            None => write!(out, "\n")?,
        }
        match self.quote.as_ref() {
            Some(quote) => {
                for line in quote.lines() {
                    write!(out, " > {}\n", line)?;
                }
            }
            None => {}
        }
        if self.cc.len() > 0 {
            let cc_list: Vec<_> = self.cc
                .iter()
                .map(|person| format!("[@{}](https://github.com/{})", person, person))
                .collect();
            write!(out, "{}\n", cc_list.join(", "))?;
        }
        Ok(())
    }
}

pub struct Formal {
    entries: HashMap<String, Entry>,
}

impl Formal {
    pub fn new() -> Formal {
        Formal {
            entries: HashMap::new(),
        }
    }

    fn parse(&mut self, yaml: &str) {
        let entry = Entry::parse(yaml);
        match entry {
            Some(e) => {
                if let Some(ent) = self.entries.get_mut(&e.name) {
                    ent.merge(e);
                    return;
                }
                self.entries.insert(e.name.clone(), e);
            }
            None => {}
        }
    }
}

impl Extractor for Formal {
    fn extract(&mut self, comment: &str) -> bool {
        let begin = Regex::new(r"```[:space:]*(yaml|yml)").unwrap();
        let end = Regex::new(r"```").unwrap();
        let mut entry = String::new();
        let mut in_yaml = false;
        let mut res = false;
        for line in comment.lines() {
            if begin.is_match(line) {
                entry = String::new();
                in_yaml = true;
            } else if end.is_match(line) {
                res = true;
                self.parse(&entry);
                in_yaml = false;
            } else if in_yaml {
                entry.push_str(line);
                entry.push_str("\n");
            }
        }
        res
    }

    fn render(&self, out: &mut io::Write) -> Result<(), Error> {
        for entry in self.entries.values() {
            entry.render(out)?;
        }
        Ok(())
    }
}
