use vaulty::{email, mailgun};

use warp::{Filter, Rejection, reply::Reply};

pub fn index() -> impl Filter<Extract = (&'static str, ), Error = Rejection> + Clone {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    warp::path::end().map(|| "Welcome to Vaulty!")
}

/// Handles mail notifications from Mailgun
pub fn mailgun() -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
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
        .and_then(|content_type: Option<String>, body: String| {
            async move {
                if let None = content_type {
                    return Err(warp::reject::not_found());
                }

                let mut mail = match mailgun::Email::from_body(&body, &content_type.unwrap()) {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("{:?}", e);
                        return Err(warp::reject::not_found());
                    }
                };

                if let Err(_e) = mail.fetch_attachments().await {
                    return Err(warp::reject::not_found());
                };

                log::info!("Fetched all attachments successfully!");

                let handler = vaulty::EmailHandler::new();
                let mail: email::Email = mail.into();

                if let Err(_e) = handler.handle(mail).await {
                    return Err(warp::reject::not_found());
                }

                log::info!("Mail handling completed");

                Ok(warp::reply())
            }
        })
}
