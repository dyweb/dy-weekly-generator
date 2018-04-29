use std::io;

use error::Error;

pub trait Extractor {
    fn extract(&mut self, &str) -> bool;
    fn render(&self, &mut io::Write) -> Result<(), Error>;
}

pub struct WeeklyBuilder {
    extractors: Vec<Box<Extractor>>,
}

pub struct Weekly {
    extractors: Vec<Box<Extractor>>,
}

impl WeeklyBuilder {
    pub fn new() -> WeeklyBuilder {
        WeeklyBuilder {
            extractors: Vec::new(),
        }
    }

    pub fn add_extractor(mut self, extractor: Box<Extractor>) -> Self {
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

    pub fn render(&self, out: &mut io::Write) -> Result<(), Error> {
        let header = r#"---
layout: post
title: Weekly
category: Weekly
author: 东岳

---

"#;
        write!(out, "{}", header)?;
        for extractor in &self.extractors {
            extractor.render(out)?
        }
        Ok(())
    }
}
