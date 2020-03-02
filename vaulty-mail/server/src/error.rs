use std::convert::Infallible;

use warp::{http::StatusCode, Rejection, Reply};

#[derive(Debug)]
pub enum Error {
    Unauthorized,
    RejectedEmail(String),
    Database(String),
    Generic(String),
}

impl warp::reject::Reject for Error {}

// TODO: Map to JSON string with more descriptive output
pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if err.is_not_found() {
        Ok(warp::reply::with_status(
            "NOT FOUND".to_string(),
            StatusCode::NOT_FOUND,
        ))
    } else if let Some(Error::Unauthorized) = err.find() {
        Ok(warp::reply::with_status(
            "AUTH REQUIRED".to_string(),
            StatusCode::UNAUTHORIZED,
        ))
    } else if let Some(Error::RejectedEmail(e)) = err.find() {
        // Invalid argument; processed gracefully on the filter side
        Ok(warp::reply::with_status(
            e.clone(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(Error::Database(e)) = err.find() {
        // Invalid argument; processed gracefully on the filter side
        Ok(warp::reply::with_status(
            e.clone(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    } else if let Some(Error::Generic(e)) = err.find() {
        Ok(warp::reply::with_status(
            e.clone(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
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
            _ => Self::Generic(err.to_string()),
        }
    }
}

impl From<sqlx::Error> for Error {
    fn from(err: sqlx::Error) -> Self {
        Self::Database(err.to_string())
    }
}
