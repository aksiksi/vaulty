use std::env;
use std::io::Read;
use std::time::Duration;

use lazy_static::lazy_static;

use reqwest::StatusCode;

use structopt::StructOpt;

mod error;

use error::Error;

// TODO: Can we make this more flexible?
lazy_static! {
    static ref VAULTY_USER: String = env::var("VAULTY_USER").expect("No auth username found!");
    static ref VAULTY_PASS: String = env::var("VAULTY_PASS").expect("No auth username found!");
}

// Request timeout, in seconds
const REQUEST_TIMEOUT: u64 = 15;

// Postfix filter error codes
// Postfix will re-queue delivery of the email to this filter
const TEMPFAIL: i32 = 75;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "vaulty-filter",
    about = "Vaulty filter for Postfix incoming mail."
)]
struct Opt {
    #[structopt(short, long)]
    sender: String,

    #[structopt(short, long)]
    recipients: Vec<String>,
}

fn send_attachment(
    remote_addr: &str,
    client: &reqwest::blocking::Client,
    email: &vaulty::email::Email,
    attachment: vaulty::email::Attachment,
) -> Result<(), Error> {
    log::info!(
        "Processing attachment for email: {}",
        attachment.get_email_id().to_string()
    );

    // Body just contains the attachment
    // All metadata passed along as headers
    let req = client
        .post(&format!("http://{}:7777/postfix/attachment", remote_addr))
        .header(reqwest::header::CONTENT_TYPE, attachment.get_mime())
        .header(reqwest::header::CONTENT_LENGTH, attachment.get_size())
        .header(vaulty::constants::VAULTY_EMAIL_ID, &email.uuid.to_string())
        .header(
            vaulty::constants::VAULTY_ATTACHMENT_NAME,
            attachment.get_name(),
        )
        .basic_auth(VAULTY_USER.as_str(), Some(VAULTY_PASS.as_str()))
        .body(reqwest::blocking::Body::from(attachment.get_data_owned()));

    let resp = req.send();
    if let Err(e) = resp {
        if e.is_timeout() {
            log::error!("Request to server timed out...: {}", e.to_string());
        }

        return Err(Error::Unexpected);
    }

    let resp = resp.unwrap();
    let body = resp.text()?;

    log::info!("{}", body);

    Ok(())
}

/// Transmit this email to the Vaulty processing server
fn process(remote_addr: &str, mail: &mut vaulty::email::Email) -> Result<(), Error> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .unwrap();
    let email = serde_json::to_string(&mail)?;

    let req = client
        .post(&format!("http://{}:7777/postfix/email", remote_addr))
        .basic_auth(VAULTY_USER.as_str(), Some(VAULTY_PASS.as_str()))
        .body(reqwest::blocking::Body::from(email));

    let resp = req.send();
    if let Err(e) = resp {
        if e.is_timeout() {
            log::error!("Request to server timed out...: {}", e.to_string());
        }

        return Err(Error::Unexpected);
    }

    let resp = resp.unwrap();

    let status = resp.status();
    let is_success = status.is_success();

    let body = &resp.text()?;

    if !is_success {
        // TODO: Handle all possible error codes
        if status == StatusCode::UNPROCESSABLE_ENTITY {
            // Reject the email gracefully
            log::info!("{}", body);
            return Err(Error::Server(body.to_string()));
        } else {
            // Unexpected server error
            log::error!("Failed to process email {} with: \"{}\"", mail.uuid, body);
            return Err(Error::Unexpected);
        }
    }

    let attachments = mail.attachments.take();

    // Send each attachment one at a time
    if let Some(attachments) = attachments {
        for a in attachments.into_iter() {
            match send_attachment(&remote_addr, &client, &mail, a) {
                Err(e) => log::error!("Failed to send this attachment: {}", e.to_string()),
                Ok(_) => (),
            }
        }
    }

    Ok(())
}

fn main() {
    let remote_addr = match env::var("VAULTY_SERVER_ADDR") {
        Ok(v) => v,
        Err(_) => "127.0.0.1".to_string(),
    };

    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    let opt = Opt::from_args();

    // Get message body from stdin
    let mut email_content = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut email_content) {
        // Message body is invalid for some reason - exit cleanly with a message
        log::error!("Failed to read message from stdin: {}", e.to_string());
        return;
    }

    // Parse and process email
    let mut mail = vaulty::email::Email::from_mime(email_content.as_bytes())
        .unwrap()
        .with_sender(opt.sender)
        .with_recipients(opt.recipients);

    // Process this email
    // If an error is encountered, we send a reply to the user
    std::process::exit(match process(&remote_addr, &mut mail) {
        Err(e) => error::reply_with_error(&mail, e),
        Ok(_) => 0,
    })
}
