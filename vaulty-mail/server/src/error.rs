use std::convert::Infallible;

use warp::{http::StatusCode, Rejection, Reply};

#[derive(Debug)]
pub enum Error {
    Unauthorized,
    InvalidRecipient,
    SenderNotWhitelisted { recipient: String },
    TokenExpired,
    QuotaExceeded(String),
    Database(String),
    Generic(String),
}

impl warp::reject::Reject for Error {}

/// Maps internal server errors to HTTP return codes.
///
/// All HTTP responses with error code `UNPROCESSABLE_ENTITY` are visible to
/// users of Vaulty. These messages are displayed to the user verbatim as part
/// of an email reply.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if err.is_not_found() {
        Ok(warp::reply::with_status(
            "NOT FOUND".to_string(),
            StatusCode::NOT_FOUND,
        ))
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::Unauthorized => Ok(warp::reply::with_status(
                "AUTH REQUIRED".to_string(),
                StatusCode::UNAUTHORIZED,
            )),
            Error::InvalidRecipient => Ok(warp::reply::with_status(
                "None of the recipients of this email are valid Vaulty addresses.".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
            Error::SenderNotWhitelisted { recipient: r } => Ok(warp::reply::with_status(
                format!("The sender of this email is not on the whitelist for address {}.", r),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
            Error::TokenExpired => Ok(warp::reply::with_status(
                "The storage account token has expired for this Vaulty address. Please login to Vaulty to refresh the token.".to_string(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
            Error::QuotaExceeded(msg) => Ok(warp::reply::with_status(
                msg.clone(),
                StatusCode::UNPROCESSABLE_ENTITY,
            )),
            Error::Database(msg) => Ok(warp::reply::with_status(
                msg.clone(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
            Error::Generic(msg) => Ok(warp::reply::with_status(
                msg.clone(),
                StatusCode::INTERNAL_SERVER_ERROR,
            )),
        }
    } else if let Some(e) = err.find::<warp::reject::MissingHeader>() {
        let header_name = e.name();

        if header_name == "Authorization" {
            Ok(warp::reply::with_status(
                format!("This endpoint requires authorization"),
                StatusCode::UNAUTHORIZED,
            ))
        } else {
            Ok(warp::reply::with_status(
                format!("Missing the following header: {}", header_name),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    } else {
        Ok(warp::reply::with_status(
            "INTERNAL SERVER ERROR".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}

impl From<vaulty::Error> for Error {
    fn from(err: vaulty::Error) -> Self {
        match err {
            vaulty::Error::Database(msg) => Self::Database(msg),
            vaulty::Error::Storage(vaulty::storage::Error::TokenExpired(_)) => Self::TokenExpired,
            _ => Self::Generic(err.to_string()),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}
