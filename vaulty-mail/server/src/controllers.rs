use bytes::{buf::Buf, Bytes};
use futures::stream::{self, FuturesUnordered, Stream, StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use serde::Serialize;
use tokio::sync::RwLock;
use warp::{self, http::Response, reply::Reply, Rejection};

use vaulty::{db::LogLevel, email, mailgun};

use super::cache::{Cache, CacheEntry};
use super::error::Error;

lazy_static! {
    /// Global mail cache
    static ref MAIL_CACHE: RwLock<Cache> = RwLock::new(Cache::new());
}

pub mod postfix {
    use super::*;

    pub async fn email(
        mut email: email::Email,
        mut db: sqlx::PgPool,
    ) -> Result<impl Reply, Rejection> {
        let mut db_client = vaulty::db::Client::new(&mut db);
        let uuid = email.uuid.to_string();

        // Build a generic success response
        let result = Response::builder()
            .body(format!("{}, {}", email.sender, uuid))
            .map_err(|e| {
                let err = Error::Generic(e.to_string());
                warp::reject::custom(err)
            });

        // Check if this email is already in the cache
        // This can occur in the case of the client retrying after a temporary
        // failure (e.g., server timeout).
        if email.num_attachments > 0 {
            // Email body has already been processed, so let's bail out here
            // Client will send us the attachments next
            if MAIL_CACHE.read().await.contains(&uuid) {
                return result;
            }
        }

        // Get address information for the relevant recipient address
        // Use this to verify that user still has enough quota remaining
        let recipients = &email.recipients.iter().map(|r| r.as_str()).collect();
        let address = match db_client.get_address(recipients).await {
            Ok(a) => a,
            Err(e) => {
                let msg = e.to_string();
                log::error!("{}", msg);
                return Err(warp::reject::custom(Error::from(e)));
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

                let err = Error::InvalidRecipient;
                return Err(warp::reject::custom(err));
            }
            Some(a) => a,
        };

        // Update the email to just have the valid recipient address
        // found above
        let recipient = &address.address;
        email.recipients.retain(|r| r == recipient);

        // Ensure that sender address is whitelisted
        let valid = address.validate_sender(&email, &mut db_client).await;
        if let Err(e) = valid {
            let msg = e.to_string();
            log::error!("{}", msg);
            return Err(warp::reject::custom(Error::from(e)));
        }

        if !valid.unwrap() {
            // Sender is not on the whitelist
            // Fail gracefully...
            log::warn!(
                "Rejecting email {:?} due to non-whitelisted sender",
                email.message_id
            );

            let err = Error::SenderNotWhitelisted {
                recipient: recipient.to_string(),
            };
            return Err(warp::reject::custom(err));
        }

        // Insert this email into DB
        if let Err(e) = db_client.insert_email(&email).await {
            let msg = e.to_string();
            log::error!("{}", msg);
            return Err(warp::reject::custom(Error::from(e)));
        }

        // Verify that address quota is not exceeded with this email
        // Quota is checked again on every attachment
        let max_email_size = address.max_email_size;
        let is_email_size_exceeded = email.size as i32 > max_email_size;
        let is_storage_quota_exceeded =
            (address.storage_used + email.size as i64) > address.storage_quota;
        let is_email_quota_exceeded = (address.num_received + 1) > address.email_quota;
        let reject = is_email_size_exceeded || is_storage_quota_exceeded || is_email_quota_exceeded;

        if reject {
            let msg = if is_email_size_exceeded {
                format!(
                    "This email is larger than allowed for {}: the maximum email size is {} MB.",
                    recipient,
                    (max_email_size / 1_000_000),
                )
            } else if is_storage_quota_exceeded {
                format!(
                    "Address {} has hit its storage quota of {} MB for this period.",
                    recipient,
                    (address.storage_quota / 1_000_000)
                )
            } else {
                format!(
                    "Address {} has hit its quota of {} emails for this period.",
                    recipient, address.email_quota,
                )
            };

            log::warn!("{}", msg);

            db_client
                .log(&msg, Some(&email.uuid), LogLevel::Warning)
                .await;

            db_client.update_email(&email, false, Some(&msg)).await;

            let err = Error::QuotaExceeded(msg);
            return Err(warp::reject::custom(err));
        }

        // Increment received storage for the email body
        // If this fails, do not proceed with processing this email
        // TODO: Can we do this in a single transaction (merge with above)?
        if let Err(e) = address
            .update_storage_used(email.body.len(), true, &mut db_client)
            .await
        {
            let msg = e.to_string();
            log::error!("{}", msg);
            return Err(warp::reject::custom(Error::from(e)));
        }

        let msg = format!("Got email for recipient {}", recipient);

        log::info!("{}", msg);
        db_client.log(&msg, Some(&email.uuid), LogLevel::Info).await;

        log::info!("{}, {}", email.sender, uuid);

        // Create a cache entry if email has attachments
        if email.num_attachments > 0 {
            log::info!("Creating cache entry for {}", email.uuid);

            let entry = CacheEntry {
                email,
                address,
                attachments_processed: Vec::new(),
                insertion_time: None,
                last_updated: None,
            };

            MAIL_CACHE.write().await.insert(uuid.clone(), entry);
        }

        result
    }

    pub async fn attachment(
        size: usize,
        _content_type: String,
        mail_id: String,
        name: String,
        index: u16,
        body: impl Stream<Item = Result<impl Buf, warp::Error>> + Send + Sync + 'static,
        mut db: sqlx::PgPool,
    ) -> Result<impl Reply, Rejection> {
        let resp = Response::builder();
        let mut db_client = vaulty::db::Client::new(&mut db);

        // Acquire cache read lock and clone email
        // This minimizes read lock time
        let entry = { MAIL_CACHE.read().await.get(&mail_id).map(|e| e.clone()) };

        // We did not find an entry for this attachment...
        if entry.is_none() {
            let msg = format!(
                "No entry found for one of the attachments (mail_id: {})",
                mail_id
            );
            let err = Error::Generic(msg);
            return Err(warp::reject::custom(err));
        }

        let entry = entry.unwrap();

        let email = &entry.email;
        let address = &entry.address;

        // Figure out if we've already processed this attachment by checking
        // the attachment index against the number of processed attachments
        // If we've processed it, silently terminate here
        if entry.attachments_processed.contains(&index) {
            let msg = format!(
                "Attachment {} has already been processed for email {}",
                index, mail_id
            );

            log::info!("{}", msg);

            return resp.body(msg).map_err(|_| warp::reject::reject());
        }

        let recipient = &email.recipients[0];
        let msg = format!("Got attachment for recipient {}", recipient);
        db_client.log(&msg, Some(&email.uuid), LogLevel::Info).await;

        log::info!(
            "Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
            name,
            recipient,
            size,
            mail_id
        );

        // Check if processing this attachment will result in the user exceeding
        // their quota. We need to check again here because another email may have been
        // processed in between (e.g., this email has been retried).
        let is_quota_exceeded = (address.storage_used + size as i64) > address.storage_quota;
        if is_quota_exceeded {
            let msg = format!(
                "Address {} has hit its quota of {} MB for this period.",
                recipient,
                (address.storage_quota / 1_000_000)
            );

            log::warn!("{}", msg);

            db_client
                .log(&msg, Some(&email.uuid), LogLevel::Warning)
                .await;

            db_client.update_email(&email, false, Some(&msg)).await;

            let err = Error::QuotaExceeded(msg);
            return Err(warp::reject::custom(err));
        }

        let resp = resp
            .body(format!(
                "Attachment name: {}, Recipient: {}, Size: {}, UUID: {}",
                name, recipient, size, mail_id
            ))
            .unwrap();

        let handler = vaulty::EmailHandler::new(
            &address.storage_token,
            &address.storage_backend,
            &address.storage_path,
        );

        let attachment = body
            .map_ok(|mut b| b.to_bytes())
            .map_err(|e| vaulty::Error::Generic(e.to_string()));

        let h = handler.handle(email, Some(attachment), name, size).await;

        // If an error occurred while processing this attachment,
        // mark the email as failed
        if let Err(e) = h.as_ref() {
            db_client
                .update_email(&email, false, Some(&e.to_string()))
                .await;
        }

        let resp = h
            .map(|_| resp)
            .map_err(|e| warp::reject::custom(Error::from(e)));

        if resp.is_ok() {
            if entry.attachments_processed.len() + 1 < email.num_attachments as usize {
                // Update the cache entry
                let mut lock = MAIL_CACHE.write().await;
                let entry = lock.get_mut(&mail_id).unwrap();
                entry.attachments_processed.push(index);
            } else {
                // If this is the last attachment for this email, cleanup the cache
                // entry.
                log::info!("Removing {} from cache", mail_id);
                MAIL_CACHE.write().await.remove(&mail_id);
            }

            // Update used storage for this attachment on success
            if let Err(e) = address
                .update_storage_used(size, false, &mut db_client)
                .await
            {
                let msg = e.to_string();
                log::error!("{}", msg);
                return Err(warp::reject::custom(Error::from(e)));
            }
        }

        resp
    }
}

/// JSON endpoints used to monitor server state
pub mod monitor {
    use super::*;

    /// Returns a snapshot of mail cache state
    pub async fn cache(mut _db: sqlx::PgPool) -> Result<impl Reply, Rejection> {
        #[derive(Serialize)]
        struct CacheState {
            num_processed: u64,
            avg_processing_time: f32,
        }

        let state = {
            let cache = MAIL_CACHE.read().await;

            CacheState {
                num_processed: cache.num_processed,
                avg_processing_time: cache.avg_processing_time,
            }
        };

        Ok(warp::reply::json(&state))
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
        .map_err(|e| vaulty::Error::Generic(e.to_string()))
        .and_then(|a| {
            let name = a.get_name().clone();
            let size = a.get_size();
            let data = vec![Ok(Bytes::from(a.get_data_owned()))];
            let data = stream::iter(data);
            handler.handle(&mail, Some(data), name, size)
        })
        .map_err(|_| warp::reject::not_found());

    // TODO: Consider making handle_email and handle_attachment
    // Compiler complains about "unknown" type for the Option
    // let email_task = handler.handle(&mail, None, "", 0).await;
    // if let Err(_) = email_task {
    //     return Err(warp::reject::not_found());
    // }

    for r in attachment_tasks
        .collect::<Vec<Result<(), warp::reject::Rejection>>>()
        .await
    {
        if let Err(_) = r {
            return Err(warp::reject::not_found());
        }
    }

    log::info!("Fetched all attachments successfully!");

    log::info!("Mail handling completed");

    Ok(warp::reply())
}
