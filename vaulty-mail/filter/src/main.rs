use std::env;
use std::io::Read;

use futures::stream::{FuturesUnordered, StreamExt};

use lazy_static::lazy_static;

use reqwest::StatusCode;

use structopt::StructOpt;

// TODO: Can we make this more flexible?
lazy_static! {
    static ref VAULTY_USER: String = env::var("VAULTY_USER").expect("No auth username found!");
    static ref VAULTY_PASS: String = env::var("VAULTY_PASS").expect("No auth username found!");
}

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

async fn send_attachment(
    remote_addr: &str,
    client: &reqwest::Client,
    email: &vaulty::email::Email,
    attachment: vaulty::email::Attachment,
) -> Result<(), Box<dyn std::error::Error>> {
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
        .body(reqwest::Body::from(attachment.get_data_owned()));

    let resp = req.send().await?;
    let bytes = &resp.bytes().await?;
    let resp_str = std::str::from_utf8(bytes)?;

    log::info!("{}", resp_str);

    Ok(())
}

/// Transmit this email to the Vaulty processing server
async fn process(
    remote_addr: &str,
    mut mail: vaulty::email::Email,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let email = serde_json::to_string(&mail)?;

    let req = client
        .post(&format!("http://{}:7777/postfix/email", remote_addr))
        .basic_auth(VAULTY_USER.as_str(), Some(VAULTY_PASS.as_str()))
        .body(reqwest::Body::from(email));

    let resp = req.send().await?;
    let status = resp.status();
    let is_success = status.is_success();

    let body = &resp.text().await?;

    if !is_success {
        if status == StatusCode::UNPROCESSABLE_ENTITY {
            // None of the listed recipients are valid
            // Reject the email gracefully
            log::info!("{}", body);
            return Ok(());
        } else {
            log::error!("Failed to process email {} with: \"{}\"", mail.uuid, body);
            return Err("Failed to process email".into());
        }
    }

    let attachments = mail.attachments.take();

    if let Some(attachments) = attachments {
        // 1. Create an iterator of `Future<Result<_, _>>`
        // 2. Collect the futures in a `FuturesUnordered`
        // 3. Collect the results into a `Vec`
        attachments
            .into_iter()
            .map(|a| send_attachment(&remote_addr, &client, &mail, a))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let remote_addr = match env::var("VAULTY_SERVER_ADDR") {
        Ok(v) => v,
        Err(_) => "127.0.0.1".to_string(),
    };

    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    let opt = Opt::from_args();

    // Get message body from stdin
    let mut email_content = String::new();
    std::io::stdin()
        .read_to_string(&mut email_content)
        .expect("Failed to read email body from stdin!");

    // Parse and process email
    let mail = vaulty::email::Email::from_mime(email_content.as_bytes())
        .unwrap()
        .with_sender(opt.sender)
        .with_recipients(opt.recipients);

    process(&remote_addr, mail).await.unwrap();
}
