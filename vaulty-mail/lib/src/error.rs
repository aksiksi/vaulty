use super::storage;

/// All possible Vaulty library errors
#[derive(Debug)]
pub enum Error {
    Generic(String),
    Database(String),
    Storage(storage::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Generic(ref msg) => write!(f, "Generic: {}", msg),
            Error::Database(ref msg) => write!(f, "Database: {}", msg),
            Error::Storage(ref e) => write!(f, "Storage: {}", e.to_string()),
        }
    }
}

impl std::error::Error for Error {}

impl From<storage::Error> for Error {
    fn from(err: storage::Error) -> Self {
        Error::Storage(err)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}
