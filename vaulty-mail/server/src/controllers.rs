use vaulty::{email, mailgun};

use warp::{Rejection, reply::Reply};

pub async fn mailgun(content_type: Option<String>, body: String) -> Result<impl Reply, Rejection> {
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
