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

         log::info!("{}, {}, {}", mail.subject, mail.sender, uuid);

         let resp =
             resp.body(format!("{}, {}, {}", mail.subject, mail.sender, uuid))
                 .map_err(|e| {
                     let err = errors::VaultyError { msg: e.to_string() };
                     warp::reject::custom(err)
                 });

         // Create a cache entry if this email has attachments
         if let Some(_) = mail.num_attachments {
             routes::MAIL_CACHE.insert(uuid.clone(), mail);
         }

         resp
    }

    pub async fn attachment(body: bytes::Bytes) -> Result<impl Reply, Rejection> {
         let resp = Response::builder();

         // TODO: No unwrap!
         let attachment: vaulty::email::Attachment
             = rmp_serde::decode::from_read(body.as_ref()).unwrap();

         let uuid = attachment.get_email_id().to_string();

         log::debug!("Got attachment for email {}", uuid);

         let is_last_attachment = {
             let mut lock = routes::MAIL_CACHE.get_mut(&uuid).unwrap();
             let mail = &mut *lock;
             let attachment_count = &mut mail.num_attachments.unwrap();

             *attachment_count -= 1;
             *attachment_count == 0
         };

         // If this is the last attachment, remove the cache entry
         if is_last_attachment {
             log::info!("Removing email {} from cache", uuid);
             routes::MAIL_CACHE.remove(&uuid);
         }

         let handler = vaulty::EmailHandler::new();
         let lock = routes::MAIL_CACHE.get(&uuid).unwrap();
         let email = &*lock;
         let recipient = &email.recipients[0];

         log::info!("Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                    attachment.get_name(), recipient, attachment.get_size(), uuid);

         let resp = resp.body(
            format!("Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                    attachment.get_name(), recipient, attachment.get_size(), uuid)
         ).unwrap();

         let result = handler.handle(email, Some(attachment)).await
               .map(|_| resp)
               .map_err(|e| {
                   let err = errors::VaultyError { msg: e.to_string() };
                   warp::reject::custom(err)
               });

         result
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
