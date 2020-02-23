use super::storage;

/// All possible Vaulty library errors
#[derive(Debug)]
pub enum Error {
    GenericError(String),
    StorageError(storage::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::GenericError(ref msg) => write!(f, "GenericError: {}", msg),
            Error::StorageError(ref e) => write!(f, "StorageError: {}", e.to_string()),
        }
    }
}

impl std::error::Error for Error {}

impl From<storage::Error> for Error {
    fn from(err: storage::Error) -> Self {
        Error::StorageError(err)
    }
}
