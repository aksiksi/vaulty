use super::errors;
use super::routes;

use vaulty::{email, mailgun};

use warp::{Rejection, http::Response, reply::Reply};

use futures::stream::{FuturesUnordered, StreamExt, TryStreamExt};

pub mod postfix {
    use super::*;

    pub async fn email(mail: email::Email) -> Result<impl Reply, Rejection> {
         let resp = Response::builder();

         let uuid = mail.uuid.to_string();

         if let Some(n) = mail.num_attachments {
             let session = routes::MailSession {
                num_attachments: n,
                recipient: mail.recipients[0].clone(),
             };

             routes::MAIL_CACHE.insert(uuid.clone(), session);
         }

         log::info!("{}, {}, {}", mail.subject, mail.sender, uuid);

         resp.body(format!("{}, {}, {}", mail.subject, mail.sender, uuid))
             .map_err(|e| {
                 let err = errors::VaultyError { msg: e.to_string() };
                 warp::reject::custom(err)
             })
    }

    pub async fn attachment(body: bytes::Bytes) -> Result<impl Reply, Rejection> {
         let resp = Response::builder();

         // TODO: No unwrap!
         let attachment: vaulty::email::Attachment
             = rmp_serde::decode::from_read(body.as_ref()).unwrap();

         let attachment = attachment.data();
         let uuid = &attachment.email_id.to_string();

         log::debug!("Got attachment for email {}", uuid);

         let recipient;

         let is_last_attachment = {
             let mut lock = routes::MAIL_CACHE.get_mut(uuid).unwrap();

             let mail_session = &mut *lock;
             recipient = mail_session.recipient.clone();

             let attachment_count = &mut mail_session.num_attachments;
             *attachment_count -= 1;

             *attachment_count == 0
         };

         // If this is the last attachment, remove the cache entry
         if is_last_attachment {
             log::info!("Removing email {} from cache", uuid);
             routes::MAIL_CACHE.remove(uuid);
         }

         log::info!("Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                    attachment.name, recipient, attachment.size, uuid);

         resp.body(
             format!("Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                    attachment.name, recipient, attachment.size, uuid)
         )
         .map_err(|e| {
             let err = errors::VaultyError { msg: e.to_string() };
             warp::reject::custom(err)
         })
    }
}

pub async fn mailgun(content_type: Option<String>, body: String,
                     api_key: Option<String>) -> Result<impl Reply, Rejection> {
    if let None = content_type {
        return Err(warp::reject::not_found());
    }

    let content_type = content_type.unwrap();

    let mail;
    let attachments;

    if content_type == "application/json" {
        mail = match mailgun::Email::from_json(&body) {
            Ok(m) => m,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(warp::reject::not_found());
            }
        };

        attachments = match mailgun::Attachment::from_json(&body) {
            Ok(m) => m,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(warp::reject::not_found());
            }
        };
    } else if content_type == "application/x-www-form-urlencoded" {
        mail = match mailgun::Email::from_form(&body) {
            Ok(m) => m,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(warp::reject::not_found());
            }
        };

        attachments = match mailgun::Attachment::from_form(&body) {
            Ok(m) => m,
            Err(e) => {
                log::error!("{:?}", e);
                return Err(warp::reject::not_found());
            }
        };
    } else {
        return Err(warp::reject::not_found());
    }

    let mail: email::Email = mail.into();
    let handler = vaulty::EmailHandler::new();

    let attachment_tasks =
        attachments.into_iter()
                   .map(|a| a.fetch(api_key.as_ref()))
                   .collect::<FuturesUnordered<_>>()
                   .map_ok(|a| email::Attachment::from(a))
                   .and_then(|a| handler.handle(&mail, Some(a)))
                   .map_err(|_| warp::reject::not_found());

    let email_task = handler.handle(&mail, None);
    if let Err(_) = email_task.await {
        return Err(warp::reject::not_found());
    }

    for r in attachment_tasks.collect::<Vec<_>>().await {
        if let Err(_) = r {
            return Err(warp::reject::not_found());
        }
    }

    log::info!("Fetched all attachments successfully!");

    log::info!("Mail handling completed");

    Ok(warp::reply())
}
