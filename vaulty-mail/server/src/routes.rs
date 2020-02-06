use warp::{Filter, Rejection, reply::Reply};

use super::controllers;

pub fn index() -> impl Filter<Extract = (&'static str, ), Error = Rejection> + Clone {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    warp::path::end().map(|| "Welcome to Vaulty!")
}

/// Route for Postfix filter
pub fn filter() -> impl Filter<Extract = (String, ), Error = Rejection> + Clone {
    // POST /filter
    warp::path("filter")
         .and(warp::path::end())
         .and(warp::body::content_length_limit(1024 * 1024 * 20))
         .and(warp::body::bytes().and_then(|body: bytes::Bytes| {
             async move {
                 std::str::from_utf8(&body)
                     .map(String::from)
                     .map_err(|_e| warp::reject::not_found())
             }
         }))
         .map(|body: String| {
             let mail: vaulty::email::Email = serde_json::from_str(&body).unwrap();
             format!("{}\n\n{:?}", mail.body, mail.uuid)
         })
}

/// Handles mail notifications from Mailgun
pub fn mailgun(api_key: Option<String>) -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    warp::path("mailgun")
         .and(warp::path::end())
         .and(warp::body::content_length_limit(1024 * 1024 * 10))
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
