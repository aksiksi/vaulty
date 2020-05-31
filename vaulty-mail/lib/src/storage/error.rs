use std::error;
use std::fmt;

use serde::{Deserialize, Serialize};

/// Error type for storage backends.
/// Each type can store a message for logging purposes.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Error {
    UrlParseError(String),
    RequestTimeout,
    RequestError(String),
    JsonParseError(String),
    BadInput(String),
    BadEndpoint(String),
    TokenExpired(String),
    RateLimited(String),
    Internal(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::UrlParseError(_) => f.write_str("UrlParseError"),
            Error::RequestTimeout => f.write_str("RequestTimeout"),
            Error::RequestError(ref msg) => f.write_str(&format!("RequestError: {}", msg)),
            Error::JsonParseError(ref msg) => f.write_str(&format!("JsonParseError: {}", msg)),
            Error::BadInput(_) => f.write_str("BadInput"),
            Error::BadEndpoint(_) => f.write_str("BadEndpoint"),
            Error::TokenExpired(_) => f.write_str("TokenExpired"),
            Error::RateLimited(_) => f.write_str("RateLimited"),
            Error::Internal(_) => f.write_str("Internal Error"),
        }
    }
}

impl error::Error for Error {}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Self::UrlParseError(err.to_string())
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::RequestTimeout
        } else {
            Self::RequestError(err.to_string())
        }
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        Self::JsonParseError(err.to_string())
    }
}
