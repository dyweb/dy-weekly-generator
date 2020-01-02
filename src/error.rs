use std::io;

use json;
use reqwest;

pub enum Error {
    ConfigErr,
    RequestErr(reqwest::Error),
    FetchErr(reqwest::blocking::Response),
    JsonParseErr,
    IOErr(io::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::RequestErr(err)
    }
}

impl From<json::Error> for Error {
    fn from(_err: json::Error) -> Error {
        Error::JsonParseErr
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOErr(err)
    }
}
