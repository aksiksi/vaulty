use serde::{Deserialize, Serialize};

/// List of supported storage backends
/// This enum needs to be kept in sync with the PGSQL enum defined in the
/// schema
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Backend {
    Dropbox,
    Gdrive,
    S3,
}

impl std::fmt::Display for Backend {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Self::Dropbox => write!(f, "Dropbox"),
            Self::Gdrive => write!(f, "GDrive"),
            Self::S3 => write!(f, "S3"),
        }
    }
}

impl From<&str> for Backend {
    fn from(s: &str) -> Self {
        if s == "dropbox" {
            Self::Dropbox
        } else if s == "gdrive" {
            Self::Gdrive
        } else if s == "s3" {
            Self::S3
        } else {
            // Default to Dropbox
            log::error!("Unknown storage backend: {}", s);
            Self::Dropbox
        }
    }
}

impl From<String> for Backend {
    fn from(s: String) -> Self {
        s.as_str().into()
    }
}
