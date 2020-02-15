use std::collections::HashMap;

use futures::stream::{FuturesUnordered, StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use warp::{Rejection, http::Response, reply::Reply};

use vaulty::{email, errors::VaultyError, mailgun, db::{LogLevel}};

use super::errors;

lazy_static! {
    // NOTE: Might make sense to include `Address` with the cache entry.
    // One advantage is we can find the target recipient address from
    // the list of recipients once, then use the `Address` info in attachments
    // processing.
    static ref MAIL_CACHE: RwLock<HashMap<String, vaulty::email::Email>> =
        RwLock::new(HashMap::new());
}

pub mod postfix {
    use super::*;

    pub async fn email(email: email::Email, mut db: sqlx::PgPool)
        -> Result<impl Reply, Rejection> {
         let mut db_client = vaulty::db::Client::new(&mut db);

         let recipient = &email.recipients[0];
         log::info!("Got email for recipient {}", recipient);

         // Get address information for the relevant recipient address
         // Use this to verify that user still has enough quota remaining
         let address = match db_client.get_address(recipient).await {
             Ok(a) => a,
             Err(e) => {
                 let err = errors::VaultyServerError { msg: e.to_string() };
                 return Err(warp::reject::custom(err));
             },
         };

         // Increment received email count at the start
         // If this fails, do not proceed with processing this email
         if let Err(e) = db_client.update_address(&address).await {
             let err = errors::VaultyServerError { msg: e.to_string() };
             return Err(warp::reject::custom(err));
         }

         // Insert this email into DB
         if let Err(e) = db_client.insert_email(&email).await {
             log::error!("Failed to insert email: {}", e.to_string());

             let err = errors::VaultyServerError { msg: e.to_string() };
             return Err(warp::reject::custom(err));
         }

         let msg = format!("Got email for recipient {}", recipient);
         db_client.log(&email.uuid, &msg, LogLevel::Info).await;

         let uuid = email.uuid.to_string();
         let resp = Response::builder();

         log::info!("{}, {}, {}", email.subject, email.sender, uuid);

         // TODO(aksiksi): Perform checks here and return HTTP error
         // if any checks failed. This will stop filter from processing
         // any attachments.
         // Update the email state if validation fails

         let result =
             resp.body(format!("{}, {}, {}", email.subject, email.sender, uuid))
                 .map_err(|e| {
                     let err = errors::VaultyServerError { msg: e.to_string() };
                     warp::reject::custom(err)
                 });

         // Create a cache entry if email has attachments
         if let Some(_) = email.num_attachments {
             let mut cache = MAIL_CACHE.write().await;
             cache.insert(uuid.clone(), email);
         }

         result
    }

    pub async fn attachment(body: bytes::Bytes, mut db: sqlx::PgPool)
        -> Result<impl Reply, Rejection> {
         let resp = Response::builder();
         let mut db_client = vaulty::db::Client::new(&mut db);

         // TODO: No unwrap!
         let attachment: vaulty::email::Attachment
             = rmp_serde::decode::from_read(body.as_ref()).unwrap();

         let uuid = attachment.get_email_id().to_string();

         // Acquire cache read lock and clone email
         // This minimizes read lock time
         let email = {
             let cache = MAIL_CACHE.read().await;
             cache.get(&uuid).unwrap().clone()
         };

         let recipient = &email.recipients[0];
         let msg = format!("Got attachment for recipient {}", recipient);
         db_client.log(&email.uuid, &msg, LogLevel::Info).await;

         // If this is the last attachment for this email, cleanup the cache
         // entry. Get this done early to minimize the chance of leaking an entry.
         {
             let mut cache = MAIL_CACHE.write().await;
             let mail = &mut cache.get_mut(&uuid).unwrap();
             let attachment_count = mail.num_attachments.as_mut().unwrap();

             *attachment_count -= 1;

             if *attachment_count == 0 {
                 log::info!("Removing email {} from cache", uuid);
                 cache.remove(&uuid);
             }
         }

         log::info!("Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                    attachment.get_name(), recipient, attachment.get_size(), uuid);

         let resp = resp.body(
            format!("Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                    attachment.get_name(), recipient, attachment.get_size(), uuid)
         ).unwrap();

         let handler = vaulty::EmailHandler::new();

         let h = handler.handle(&email, Some(attachment)).await;

         if let Err(e) = h.as_ref() {
             db_client.update_email(&email,
                                    false,
                                    Some(&e.to_string())).await;
         }

         let resp = h.map(|_| resp)
                     .map_err(|e| {
                        let err = errors::VaultyServerError { msg: e.to_string() };
                        warp::reject::custom(err)
                     });

         // TODO: If result contains an error, log this to DB

         resp
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
                   .map_err(|e| VaultyError { msg: e.to_string() })
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
