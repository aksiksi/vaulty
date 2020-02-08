use std::sync::atomic;

use chashmap::CHashMap;
use lazy_static::lazy_static;
use warp::{Filter, Rejection, http::Response, reply::Reply};

use super::controllers;

struct MailSession {
    recipient: String,
    num_attachments: atomic::AtomicU32,
}

const MAX_EMAIL_SIZE: u64 = 5 * 1024 * 1024;
const MAX_ATTACHMENT_SIZE: u64 = 20 * 1024 * 1024;

lazy_static! {
    static ref MAIL_CACHE: CHashMap<String, MailSession> = CHashMap::new();
}

pub fn index() -> impl Filter<Extract = (&'static str, ), Error = Rejection> + Clone {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    warp::path::end().map(|| "Welcome to Vaulty!")
}

/// Route for Postfix email
pub fn email() -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    warp::path!("postfix" / "email")
         .and(warp::path::end())
         .and(warp::header::<String>("Authorization"))
         .and(warp::body::content_length_limit(MAX_EMAIL_SIZE))
         .and(warp::body::json())
         .map(|auth: String, mail: vaulty::email::Email| {
             let resp = Response::builder();

             if !auth.contains("TEST123") {
                 return resp.status(403).body("".to_string());
             }

             let uuid = mail.uuid.to_string();

             if let Some(n) = mail.num_attachments {
                 let session = MailSession {
                    num_attachments: atomic::AtomicU32::new(n),
                    recipient: mail.recipients[0].clone(),
                 };

                 MAIL_CACHE.insert(uuid.clone(), session);
             }

             log::info!("{}, {}, {}", mail.subject, mail.sender, uuid);

             resp.body(format!("{}, {}, {}", mail.subject, mail.sender, uuid))
         })
}

/// Route for Postfix attachment
pub fn attachment() -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    warp::path!("postfix" / "attachment")
         .and(warp::path::end())
         .and(warp::body::content_length_limit(MAX_ATTACHMENT_SIZE))
         .and(warp::body::bytes())
         .and(warp::header::<String>("Authorization"))
         .map(|body: bytes::Bytes, auth: String| {
             let resp = Response::builder();

             if !auth.contains("TEST123") {
                 return resp.status(403).body("".to_string());
             }

             let attachment: vaulty::email::Attachment
                 = rmp_serde::decode::from_read(body.as_ref()).unwrap();
             let attachment = attachment.data();

             let uuid = &attachment.email_id.to_string();

             // TODO: Handle malicious requests (do not unwrap!)
             let mail_session = &*MAIL_CACHE.get_mut(uuid).unwrap();
             let recipient = mail_session.recipient.clone();
             let attachment_count = &mail_session.num_attachments;

             // If this is the last attachment, remove the cache entry
             if attachment_count.fetch_sub(1, atomic::Ordering::SeqCst) == 0 {
                 log::info!("Removing {} from cache", uuid);
                 MAIL_CACHE.remove(uuid);
             }

             log::info!("Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                        attachment.name, recipient, attachment.size, uuid);

             resp.body(
                 format!("Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                        attachment.name, recipient, attachment.size, uuid)
             )
         })
}

/// Handles mail notifications from Mailgun
pub fn mailgun(api_key: Option<String>) -> impl Filter<Extract = (impl Reply, ), Error = Rejection> + Clone {
    warp::path("mailgun")
         .and(warp::path::end())
         .and(warp::body::content_length_limit(MAX_EMAIL_SIZE))
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
