use std::io;

use regex::Regex;

use error::Error;
use weekly::Extractor;

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
        let url_pattern = Regex::new(r"(((http://)|(https://)|(ftp://)|(www\.))([a-zA-Z0-9_\-]+\.)*[a-zA-Z0-9\-]+\.[a-zA-Z0-9\-]+)").unwrap();
        let res = url_pattern.is_match(comment);
        if res {
            self.entries.push(comment.to_owned())
        }
        res
    }

    fn render(&self, out: &mut io::Write) -> Result<(), Error> {
        for entry in &self.entries {
            // Add a horizontal line between the entries.
            write!(out, "{}\n***\n", entry)?
        }
        Ok(())
    }
}
