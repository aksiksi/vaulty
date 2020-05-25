use std::sync::Arc;

use warp::{http::header, reply::Reply, Filter, Rejection};

use super::controllers;
use super::filters;

use vaulty::config::Config;

pub fn index() -> impl Filter<Extract = (&'static str,), Error = Rejection> + Clone {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    warp::path::end().map(|| "Welcome to Vaulty!")
}

/// Route for /postfix
pub fn postfix(
    db: sqlx::PgPool,
    config: Arc<Config>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    email(db.clone(), config.clone()).or(attachment(db.clone(), config.clone()))
}

/// Route for /postfix/email
/// Handles email body and creates a cache entry to track attachments
pub fn email(
    db: sqlx::PgPool,
    config: Arc<Config>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("postfix" / "email")
        .and(warp::path::end())
        .and(warp::body::content_length_limit(config.max_email_size))
        .and(filters::basic_auth(config))
        .and(warp::body::json())
        .and_then(move |email| controllers::postfix::email(email, db.clone()))
}

/// Route for /postfix/attachment
/// Handles each email attachment
pub fn attachment(
    db: sqlx::PgPool,
    config: Arc<Config>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("postfix" / "attachment")
        .and(warp::path::end())
        .and(warp::body::content_length_limit(config.max_attachment_size))
        .and(filters::basic_auth(config))
        .and(warp::filters::header::header::<usize>(
            header::CONTENT_LENGTH.as_str(),
        ))
        .and(warp::filters::header::header::<String>(
            header::CONTENT_TYPE.as_str(),
        ))
        .and(warp::filters::header::header::<String>(
            vaulty::constants::VAULTY_EMAIL_ID,
        ))
        .and(warp::filters::header::header::<String>(
            vaulty::constants::VAULTY_ATTACHMENT_NAME,
        ))
        .and(warp::filters::header::header::<u16>(
            vaulty::constants::VAULTY_ATTACHMENT_INDEX,
        ))
        .and(warp::filters::body::stream())
        .and_then(move |size, content_type, mail_id, name, index, body| {
            controllers::postfix::attachment(
                size,
                content_type,
                mail_id,
                name,
                index,
                body,
                db.clone(),
            )
        })
}

/// Route for /monitor
pub fn monitor(
    db: sqlx::PgPool,
    config: Arc<Config>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    cache(db.clone(), config.clone())
}

/// Route for /monitor/cache
pub fn cache(
    db: sqlx::PgPool,
    _config: Arc<Config>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path!("monitor" / "cache")
        .and(warp::path::end())
        .and_then(move || controllers::monitor::cache(db.clone()))
}

/// Handles mail notifications from Mailgun
pub fn mailgun(
    config: Arc<Config>,
) -> impl Filter<Extract = (impl Reply,), Error = Rejection> + Clone {
    warp::path("mailgun")
        .and(warp::path::end())
        .and(warp::body::content_length_limit(
            vaulty::config::MAX_EMAIL_SIZE,
        ))
        .and(warp::header::optional::<String>("content-type"))
        .and(
            warp::body::bytes().and_then(|body: bytes::Bytes| async move {
                std::str::from_utf8(&body)
                    .map(String::from)
                    .map_err(|_e| warp::reject::not_found())
            }),
        )
        .and_then(move |content_type, body| {
            controllers::mailgun(content_type, body, config.mailgun_key.clone())
        })
}
