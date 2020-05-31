use super::storage;

use serde::{Deserialize, Serialize};

/// All possible Vaulty library errors
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Error {
    Generic(String),
    Database(String),
    Storage(storage::Error),
    QuotaExceeded(String),
    TokenExpired,
    InvalidRecipient,
    SenderNotWhitelisted { recipient: String },
    Unauthorized,
    NotFound,
    MissingHeader(String),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Generic(ref msg) => write!(f, "{}", msg),
            Error::Database(ref msg) => write!(f, "{}", msg),
            Error::Storage(ref e) => write!(f, "Storage error: {}", e.to_string()),
            Error::QuotaExceeded(ref msg) => write!(f, "{}", msg),
            Error::TokenExpired => write!(f, "The storage account token has expired for this Vaulty address. Please login to Vaulty to refresh the token."),
            Error::InvalidRecipient => write!(f, "None of the recipients of this email are valid Vaulty addresses."),
            Error::SenderNotWhitelisted { ref recipient } =>
                write!(f, "The sender of this email is not on the whitelist for address {}.", recipient),
            Error::Unauthorized => write!(f, "Access to this endpoint is not authorized."),
            Error::NotFound => write!(f, "No such endpoint exists."),
            Error::MissingHeader(ref msg) => {
                if msg == "Authorization" {
                    write!(f, "This endpoint requires HTTP authorization.")
                } else {
                    write!(f, "The request is missing the following header(s): {}", msg)
                }
            }
        }
    }
}

impl std::error::Error for Error {}

impl From<storage::Error> for Error {
    fn from(err: storage::Error) -> Self {
        match err {
            storage::Error::TokenExpired(_) => Self::TokenExpired,
            _ => Error::Storage(err),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}
