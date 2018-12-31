use std::io;

use lazy_static::lazy_static;
use regex::Regex;

use crate::error::Error;
use crate::weekly::Extractor;

pub struct Casual {
    entries: Vec<String>,
}

impl Casual {
    pub fn new() -> Casual {
        Casual {
            entries: Vec::new(),
        }
    }
}

impl Extractor for Casual {
    fn extract(&mut self, comment: &str) -> bool {
        lazy_static! {
            static ref URL_PATTERN: Regex = Regex::new(r"(((http://)|(https://)|(ftp://)|(www\.))([a-zA-Z0-9_\-]+\.)*[a-zA-Z0-9\-]+\.[a-zA-Z0-9\-]+)").unwrap();
        }
        let res = URL_PATTERN.is_match(comment);
        if res {
            self.entries.push(comment.to_owned())
        }
        res
    }

    fn render(&self, out: &mut dyn io::Write) -> Result<(), Error> {
        for entry in &self.entries {
            // Add a horizontal line between the entries.
            write!(out, "{}\n\n***\n\n", entry)?
        }
        Ok(())
    }
}
