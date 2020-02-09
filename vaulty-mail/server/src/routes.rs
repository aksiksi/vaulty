
use warp::{Filter, Rejection, reply::Reply};

use super::config;
use super::controllers;
use super::filters;

pub fn index() -> impl Filter<Extract = (&'static str, ), Error = Rejection> + Clone {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    warp::path::end().map(|| "Welcome to Vaulty!")
}

/// Route for /postfix
pub fn postfix() -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    email().or(attachment())
}

/// Route for /postfix/email
/// Handles email body and creates a cache entry to track attachments
pub fn email() -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    warp::path!("postfix" / "email")
         .and(warp::path::end())
         .and(filters::basic_auth())
         .and(warp::body::content_length_limit(config::MAX_EMAIL_SIZE))
         .and(warp::body::json())
         .and_then(controllers::postfix::email)
}

/// Route for /postfix/attachment
/// Handles each email attachment
pub fn attachment() -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    warp::path!("postfix" / "attachment")
         .and(warp::path::end())
         .and(filters::basic_auth())
         .and(warp::body::content_length_limit(config::MAX_ATTACHMENT_SIZE))
         .and(warp::body::bytes())
         .and_then(controllers::postfix::attachment)
}

/// Handles mail notifications from Mailgun
pub fn mailgun(api_key: Option<String>) -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    warp::path("mailgun")
         .and(warp::path::end())
         .and(warp::body::content_length_limit(config::MAX_EMAIL_SIZE))
         .and(warp::header::optional::<String>("content-type"))
         .and(warp::body::bytes().and_then(|body: bytes::Bytes| {
             async move {
                 std::str::from_utf8(&body)
                     .map(String::from)
                     .map_err(|_e| warp::reject::not_found())
             }
         }))
         .and_then(move |content_type, body| {
             controllers::mailgun(content_type, body, api_key.clone())
         })
}
