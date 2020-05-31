use std::convert::Infallible;

use warp::{http::StatusCode, Rejection, Reply};

/// Wrap the shared Vaulty error type so Reject can be impl'd
#[derive(Debug)]
pub struct Error(pub vaulty::Error);

impl warp::reject::Reject for Error {}

/// Maps internal server errors to HTTP return codes.
///
/// All HTTP responses with error code `UNPROCESSABLE_ENTITY` are visible to
/// users of Vaulty. These messages are displayed to the user verbatim as part
/// of an email reply.
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    let status_code;
    let error;

    if err.is_not_found() {
        status_code = StatusCode::NOT_FOUND;
        error = vaulty::Error::NotFound;
    } else if let Some(e) = err.find::<Error>() {
        error = e.0.clone();

        match error {
            vaulty::Error::Generic(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
            }
            vaulty::Error::Database(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
            }
            vaulty::Error::Storage(_) => {
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
            }
            vaulty::Error::QuotaExceeded(_) => {
                status_code = StatusCode::UNPROCESSABLE_ENTITY;
            }
            vaulty::Error::TokenExpired => {
                status_code = StatusCode::UNPROCESSABLE_ENTITY;
            }
            vaulty::Error::InvalidRecipient => {
                status_code = StatusCode::UNPROCESSABLE_ENTITY;
            }
            vaulty::Error::SenderNotWhitelisted { .. } => {
                status_code = StatusCode::UNPROCESSABLE_ENTITY;
            }
            vaulty::Error::Unauthorized => {
                status_code = StatusCode::UNAUTHORIZED;
            }
            _ => {
                // All other error variants are not expected here
                status_code = StatusCode::INTERNAL_SERVER_ERROR;
            }
        }
    } else if let Some(e) = err.find::<warp::reject::MissingHeader>() {
        status_code = StatusCode::UNAUTHORIZED;
        error = vaulty::Error::MissingHeader(e.name().to_string());
    } else {
        status_code = StatusCode::INTERNAL_SERVER_ERROR;
        error = vaulty::Error::Generic("Internal server error".to_string());
    }

    let resp = vaulty::api::ServerResult {
        success: false,
        error: Some(error),
        ..Default::default()
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&resp),
        status_code,
    ))
}

impl From<vaulty::Error> for Error {
    fn from(err: vaulty::Error) -> Self {
        Self(err)
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self(err.into())
    }
}
