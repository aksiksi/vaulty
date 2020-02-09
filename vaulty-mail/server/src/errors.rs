use std::convert::Infallible;

use warp::{Rejection, Reply, http::StatusCode};

#[derive(Debug)]
pub struct Unauthorized;

impl warp::reject::Reject for Unauthorized {}

#[derive(Debug)]
pub struct VaultyError {
    pub msg: String,
}

impl warp::reject::Reject for VaultyError {}

pub async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    if err.is_not_found() {
        Ok(warp::reply::with_status("NOT FOUND".to_string(),
                                    StatusCode::NOT_FOUND))
    } else if let Some(Unauthorized) = err.find() {
        Ok(warp::reply::with_status("AUTH REQUIRED".to_string(),
                                    StatusCode::UNAUTHORIZED))
    } else if let Some(VaultyError { msg: e }) = err.find() {
        Ok(warp::reply::with_status(e.clone(),
                                    StatusCode::UNAUTHORIZED))
    } else {
        Ok(warp::reply::with_status("INTERNAL SERVER ERROR".to_string(),
                                    StatusCode::INTERNAL_SERVER_ERROR))
    }
}
