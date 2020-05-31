#[derive(Debug)]
pub enum Error {
    Server(vaulty::api::ServerResult),
    Temporary,
    Unexpected,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Server(result) => {
                // Try to log underlying error verbatim
                if let Some(err) = &result.error {
                    write!(f, "{}", err.to_string())
                } else {
                    write!(f, "{:?}", result)
                }
            }
            Error::Temporary => write!(f, "Mail processing failed temporarily."),
            Error::Unexpected => write!(
                f,
                "An unexpected error occurred while processing this email.\n\n
                 Please contact Vaulty support: https://groups.google.com/forum/#!forum/vaulty-support"
            ),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(_err: reqwest::Error) -> Self {
        Self::Temporary
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(_err: serde_json::error::Error) -> Self {
        Self::Unexpected
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_err: std::str::Utf8Error) -> Self {
        Self::Unexpected
    }
}
