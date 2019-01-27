use std::io;

use crate::error::Error;

pub trait Extractor {
    fn extract(&mut self, _: &str) -> bool;
    fn render(&self, _: &mut dyn io::Write) -> Result<(), Error>;
}

pub struct WeeklyBuilder {
    extractors: Vec<Box<dyn Extractor>>,
}

pub struct Weekly {
    extractors: Vec<Box<dyn Extractor>>,
}

impl WeeklyBuilder {
    pub fn new() -> WeeklyBuilder {
        WeeklyBuilder {
            extractors: Vec::new(),
        }
    }

    pub fn add_extractor(mut self, extractor: Box<dyn Extractor>) -> Self {
        self.extractors.push(extractor);
        self
    }

    pub fn build(self) -> Weekly {
        Weekly {
            extractors: self.extractors,
        }
    }
}

impl Weekly {
    pub fn parse(&mut self, comment: &str) {
        for extractor in &mut self.extractors {
            if extractor.extract(comment) {
                break;
            }
        }
    }

    pub fn render(&self, out: &mut dyn io::Write) -> Result<(), Error> {
        let header = include_str!("weekly_header.md");
        write!(out, "{}", header)?;
        for extractor in &self.extractors {
            extractor.render(out)?
        }
        Ok(())
    }
}
