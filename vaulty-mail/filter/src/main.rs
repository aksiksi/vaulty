use std::env;
use std::io::Read;
use std::time::Duration;

use lazy_static::lazy_static;

use reqwest::StatusCode;

use structopt::StructOpt;

mod error;
mod reply;

use error::Error;

use vaulty::api::ServerResult;

// TODO: Can we make this more flexible?
lazy_static! {
    static ref VAULTY_USER: String = env::var("VAULTY_USER").expect("No auth username found!");
    static ref VAULTY_PASS: String = env::var("VAULTY_PASS").expect("No auth username found!");
}

// Request timeout, in seconds
const REQUEST_TIMEOUT: u64 = 15;

// Postfix filter error codes
// Postfix will re-queue delivery of the email to this filter
// See: https://github.com/vdukhovni/postfix/blob/bfff4380a3b6fac2513c73531ee3a79212c08660/postfix/src/global/sys_exits.h#L31
const UNAVAILABLE: i32 = 69;
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
) -> Result<ServerResult, Error> {
    log::debug!(
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
        .header(
            vaulty::constants::VAULTY_ATTACHMENT_INDEX,
            attachment.get_index(),
        )
        .basic_auth(VAULTY_USER.as_str(), Some(VAULTY_PASS.as_str()))
        .body(attachment.get_data_owned());

    let resp = req.send();
    if let Err(e) = resp {
        if e.is_timeout() {
            log::error!("Request to server timed out...: {}", e.to_string());
        }

        return Err(Error::Temporary);
    }

    let resp = resp.unwrap();
    let result = resp.json::<ServerResult>()?;

    log::debug!("{:?}", result);

    Ok(result)
}

/// Transmit this email to the Vaulty processing server
fn process(remote_addr: &str, mail: &mut vaulty::email::Email) -> Result<ServerResult, Error> {
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

        return Err(Error::Temporary);
    }

    let resp = resp.unwrap();

    let status = resp.status();
    let is_success = status.is_success();

    let mut result = resp.json::<ServerResult>()?;

    if !is_success {
        // TODO: Handle all possible error codes
        if status == StatusCode::UNPROCESSABLE_ENTITY {
            // Reject the email gracefully
            log::debug!("{:?}", result);
            return Err(Error::Server(result));
        } else {
            // Unexpected server error
            log::debug!(
                "Failed to process email {} with: \"{:?}\"",
                mail.uuid,
                result
            );
            return Err(Error::Unexpected);
        }
    }

    let attachments = mail.attachments.take();

    // Send each attachment one at a time
    if let Some(attachments) = attachments {
        let num_attachments = attachments.len();

        for (i, a) in attachments.into_iter().enumerate() {
            match send_attachment(&remote_addr, &client, &mail, a) {
                Err(e) => return Err(e),
                Ok(r) => {
                    if i == num_attachments - 1 {
                        // The last attachment gets the final result
                        result = r;
                    }
                }
            }
        }
    }

    Ok(result)
}

fn main() {
    let remote_addr = match env::var("VAULTY_SERVER_ADDR") {
        Ok(v) => v,
        Err(_) => "127.0.0.1".to_string(),
    };

    let reply_on_success = match env::var("VAULTY_REPLY_SUCCESS") {
        Ok(_) => true,
        Err(_) => false,
    };

    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    // Parse input arguments
    let opt = Opt::from_args();

    // If this is a delivery status notification (DSN), just ignore it
    // See: Postfix pipe null_sender argument
    if opt.sender == "" {
        log::warn!("Received a bounced email notification... ignoring");
        std::process::exit(0);
    }

    // Get message body from stdin
    let mut email_content = String::new();
    if let Err(_) = std::io::stdin().read_to_string(&mut email_content) {
        // Message body is invalid for some reason - exit cleanly with a message
        // NOTE(aksiksi): When providing DSN status code to Postfix, the code
        // must end with either a space or EOF.
        // See: https://github.com/vdukhovni/postfix/blob/bfff4380a3b6fac2513c73531ee3a79212c08660/postfix/src/global/dsn_util.c#L127
        println!("5.6.0 Failed to read mail body");
        std::process::exit(UNAVAILABLE);
    }

    // Try to parse this email
    let result = vaulty::email::Email::from_mime(email_content.as_bytes());
    if let Err(_) = result {
        println!("5.6.0 Failed to parse mail body");
        std::process::exit(UNAVAILABLE);
    }

    let mut mail = result
        .unwrap()
        .with_sender(opt.sender)
        .with_recipients(opt.recipients);

    // Process this email
    // If an error is encountered, we send a reply to the user
    std::process::exit(match process(&remote_addr, &mut mail) {
        Err(e) => reply::reply_error(e),
        Ok(r) => {
            if reply_on_success {
                reply::reply_success(&mail, r)
            } else {
                0
            }
        }
    })
}
