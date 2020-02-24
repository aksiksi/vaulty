use std::collections::HashMap;

use futures::stream::{FuturesUnordered, StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use tokio::sync::RwLock;
use warp::{
    http::{self, Response},
    reply::Reply,
    Rejection,
};

use vaulty::{db::LogLevel, email, mailgun};

use super::errors;

// Cache entry is cloneable to reduce read lock hold time
#[derive(Clone)]
struct CacheEntry {
    pub email: email::Email,
    pub address: vaulty::db::Address,
}

lazy_static! {
    static ref MAIL_CACHE: RwLock<HashMap<String, CacheEntry>> = RwLock::new(HashMap::new());
}

pub mod postfix {
    use super::*;

    pub async fn email(
        mut email: email::Email,
        mut db: sqlx::PgPool,
    ) -> Result<impl Reply, Rejection> {
        let mut db_client = vaulty::db::Client::new(&mut db);

        // Get address information for the relevant recipient address
        // Use this to verify that user still has enough quota remaining
        let recipients = &email.recipients.iter().map(|r| r.as_str()).collect();
        let address = match db_client.get_address(recipients).await {
            Ok(a) => a,
            Err(e) => {
                let msg = e.to_string();
                log::error!("{}", msg);
                let err = errors::VaultyServerError { msg: msg };

                return Err(warp::reject::custom(err));
            }
        };

        // If none of the recipients are valid, reject this email gracefully
        // with a unique status code.
        // NOTE: This case should never be hit as Postfix is looking at the
        // same DB.
        let address = match address {
            None => {
                // We do not use internal UUID here b/c there really is no
                // history maintained for this email.  Using Message-ID will
                // at least help with user queries as to why their email
                // never arrived.
                let msg = format!(
                    "Rejecting email message_id: {}, \
                                    from: {}, to: {}",
                    &email.message_id.unwrap_or("N/A".to_string()),
                    &email.sender,
                    &email.recipients.join(", ")
                );

                log::warn!("{}", msg);
                db_client.log(&msg, None, LogLevel::Warning).await;

                let status = http::status::StatusCode::UNPROCESSABLE_ENTITY;
                let resp = Response::builder().status(status).body(msg);

                return Ok(resp.unwrap());
            }
            Some(a) => a,
        };

        // Ensure that sender address is whitelisted
        // We scope this to avoid compiler nags about dyn Error
        {
            let valid = db_client.validate_sender_address(&address, &email).await;

            if let Err(e) = valid {
                let msg = e.to_string();
                log::error!("{}", msg);
                let err = errors::VaultyServerError { msg: msg };
                return Err(warp::reject::custom(err));
            }

            if !valid.unwrap() {
                // Sender is not on the whitelist
                // Fail gracefully...
                let msg = "Rejecting email due to non-whitelisted sender".to_string();
                let status = http::status::StatusCode::UNPROCESSABLE_ENTITY;
                let resp = Response::builder().status(status).body(msg);
                return Ok(resp.unwrap());
            }
        }

        // Update the email to just have the valid recipient address
        // found above
        let recipient = &address.address;
        email.recipients.retain(|r| r == recipient);

        // Insert this email into DB
        if let Err(e) = db_client.insert_email(&email).await {
            let msg = e.to_string();
            log::error!("{}", msg);
            let err = errors::VaultyServerError { msg: msg };

            return Err(warp::reject::custom(err));
        }

        // Verify that address quota is not exceeded with this email
        let max_email_size = address.max_email_size as usize;
        let is_mail_size_exceeded = email.size > max_email_size;
        let is_quota_exceeded = (address.received + 1) > address.quota;
        let reject = is_quota_exceeded || is_mail_size_exceeded;

        if reject {
            let msg = if is_mail_size_exceeded {
                format!(
                    "Email {} is larger than allowed for this address: {}",
                    &email.uuid, recipient
                )
            } else {
                format!("Address {} has hit its quota for this period", recipient)
            };

            log::warn!("{}", msg);

            db_client
                .log(&msg, Some(&email.uuid), LogLevel::Warning)
                .await;

            db_client.update_email(&email, false, Some(&msg)).await;

            let status = http::status::StatusCode::UNPROCESSABLE_ENTITY;
            let resp = Response::builder().status(status).body(msg);
            return Ok(resp.unwrap());
        }

        // Increment received email count at the start
        // If this fails, do not proceed with processing this email
        // TODO: Can we do this in a single transaction (merge with above)?
        if let Err(e) = db_client.update_address_received_count(&address).await {
            let msg = e.to_string();
            log::error!("{}", msg);
            let err = errors::VaultyServerError { msg: msg };

            return Err(warp::reject::custom(err));
        }

        let msg = format!("Got email for recipient {}", recipient);

        log::info!("{}", msg);
        db_client.log(&msg, Some(&email.uuid), LogLevel::Info).await;

        let uuid = email.uuid.to_string();
        let resp = Response::builder();

        log::info!("{}, {}", email.sender, uuid);

        // TODO(aksiksi): Perform checks here and return HTTP error
        // if any checks failed. This will stop filter from processing
        // any attachments.
        // Update the email state if validation fails

        let result = resp
            .body(format!("{}, {}", email.sender, uuid))
            .map_err(|e| {
                let err = errors::VaultyServerError { msg: e.to_string() };
                warp::reject::custom(err)
            });

        // Create a cache entry if email has attachments
        if let Some(_) = email.num_attachments {
            log::info!("Creating cache entry for {}", email.uuid);

            let entry = CacheEntry {
                email: email,
                address: address,
            };

            let mut cache = MAIL_CACHE.write().await;
            cache.insert(uuid.clone(), entry);
        }

        result
    }

    pub async fn attachment(
        body: bytes::Bytes,
        mut db: sqlx::PgPool,
    ) -> Result<impl Reply, Rejection> {
        let resp = Response::builder();
        let mut db_client = vaulty::db::Client::new(&mut db);

        // TODO: Perhaps make this raw instead of MessagePack
        let attachment: vaulty::email::Attachment =
            match rmp_serde::decode::from_read(body.as_ref()) {
                Err(e) => {
                    let msg = e.to_string();
                    log::error!("{}", msg);
                    let err = errors::VaultyServerError { msg: msg };
                    return Err(warp::reject::custom(err));
                }
                Ok(v) => v,
            };

        let uuid = attachment.get_email_id().to_string();

        // Acquire cache read lock and clone email
        // This minimizes read lock time
        let entry = {
            let cache = MAIL_CACHE.read().await;
            cache.get(&uuid).unwrap().clone()
        };

        let email = &entry.email;
        let address = &entry.address;

        let recipient = &email.recipients[0];
        let msg = format!("Got attachment for recipient {}", recipient);
        db_client.log(&msg, Some(&email.uuid), LogLevel::Info).await;

        // If this is the last attachment for this email, cleanup the cache
        // entry. Get this done early to minimize the chance of leaking an entry.
        {
            let mut cache = MAIL_CACHE.write().await;
            let e = &mut cache.get_mut(&uuid).unwrap();
            let attachment_count = e.email.num_attachments.as_mut().unwrap();

            *attachment_count -= 1;

            if *attachment_count == 0 {
                log::info!("Removing {} from cache", uuid);
                cache.remove(&uuid);
            }
        }

        log::info!(
            "Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
            attachment.get_name(),
            recipient,
            attachment.get_size(),
            uuid
        );

        let resp = resp
            .body(format!(
                "Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                attachment.get_name(),
                recipient,
                attachment.get_size(),
                uuid
            ))
            .unwrap();

        let handler = vaulty::EmailHandler::new(
            &address.storage_token,
            &address.storage_backend,
            &address.storage_path,
        );

        let h = handler.handle(email, Some(attachment)).await;

        if let Err(e) = h.as_ref() {
            db_client
                .update_email(&email, false, Some(&e.to_string()))
                .await;
        }

        let resp = h.map(|_| resp).map_err(|e| {
            let err = errors::VaultyServerError { msg: e.to_string() };
            warp::reject::custom(err)
        });

        // TODO: If result contains an error, log this to DB

        resp
    }
}

pub async fn mailgun(
    content_type: Option<String>,
    body: String,
    api_key: Option<String>,
) -> Result<impl Reply, Rejection> {
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
    let storage_backend: vaulty::storage::Backend = "dropbox".into();

    let handler = vaulty::EmailHandler::new("test123", &storage_backend, "/vaulty");

    let attachment_tasks = attachments
        .into_iter()
        .map(|a| a.fetch(api_key.as_ref()))
        .collect::<FuturesUnordered<_>>()
        .map_ok(|a| email::Attachment::from(a))
        .map_err(|e| vaulty::Error::GenericError(e.to_string()))
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
