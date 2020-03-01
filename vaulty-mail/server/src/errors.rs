use std::convert::Infallible;

use warp::{http::StatusCode, Rejection, Reply};

#[derive(Debug)]
pub enum Error {
    Unauthorized,
    InvalidArg(String),
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
    } else if let Some(Error::InvalidArg(e)) = err.find() {
        // Invalid argument; processed gracefully on the filter side
        Ok(warp::reply::with_status(
            e.clone(),
            StatusCode::UNPROCESSABLE_ENTITY,
        ))
    } else if let Some(Error::Generic(e)) = err.find() {
        Ok(warp::reply::with_status(
            e.clone(),
            StatusCode::UNAUTHORIZED,
        ))
    } else {
        Ok(warp::reply::with_status(
            "INTERNAL SERVER ERROR".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }
}
