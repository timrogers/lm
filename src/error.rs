use std::fmt;

#[derive(Debug)]
pub enum Error {
    Config(String),
    Auth(String),
    Api(String),
    Http(reqwest::Error),
    Io(std::io::Error),
    Yaml(serde_yaml::Error),
    Json(serde_json::Error),
    Other(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(msg) => write!(f, "Configuration error: {}", msg),
            Error::Auth(msg) => write!(f, "Authentication error: {}", msg),
            Error::Api(msg) => write!(f, "API error: {}", msg),
            Error::Http(err) => write!(f, "HTTP error: {}", err),
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Yaml(err) => write!(f, "YAML parsing error: {}", err),
            Error::Json(err) => write!(f, "JSON parsing error: {}", err),
            Error::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Self {
        Error::Yaml(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;