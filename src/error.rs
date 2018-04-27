use reqwest;

pub enum Error {
    ConfigErr,
    RequestErr(reqwest::Error),
    FetchErr,
    JsonParseErr,
    IOErr,
}
