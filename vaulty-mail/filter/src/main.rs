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
    url: &str,
    client: &reqwest::Client,
    attachment: &vaulty::email::Attachment,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info!(
        "Processing attachment for email: {}",
        attachment.get_email_id().to_string()
    );

    let raw = rmp_serde::encode::to_vec_named(&attachment)?;

    let req = client
        .post(&format!("{}/postfix/attachment", url))
        .header(reqwest::header::CONTENT_TYPE, attachment.get_mime())
        .basic_auth(VAULTY_USER.as_str(), Some(VAULTY_PASS.as_str()))
        .body(reqwest::Body::from(raw));

    let resp = req.send().await?;
    let bytes = &resp.bytes().await?;
    let resp_str = std::str::from_utf8(bytes)?;

    log::info!("{}", resp_str);

    Ok(())
}

/// Transmit this email to the Vaulty processing server
async fn process(
    remote_addr: &str,
    mail: vaulty::email::Email,
    use_tls: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = if use_tls {
        format!("https://{}:7777", remote_addr)
    } else {
        format!("http://{}:7777", remote_addr)
    };

    let client = reqwest::ClientBuilder::new()
        .use_rustls_tls()
        .build()
        .unwrap();

    let email = serde_json::to_string(&mail)?;

    let req = client
        .post(&format!("{}/postfix/email", url))
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

    if let Some(attachments) = &mail.attachments {
        // 1. Create an iterator of `Future<Result<_, _>>`
        // 2. Collect the futures in a `FuturesUnordered`
        // 3. Collect the results into a `Vec`
        attachments
            .iter()
            .map(|a| send_attachment(&url, &client, a))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    // Determine if TLS is enabled using config lookup
    // TODO: Load more stuff via config?
    let config = vaulty::config::load_config(None);
    let use_tls = config
        .get("use_tls")
        .and_then(|c| c.parse::<bool>().ok())
        .unwrap_or(false);

    let server_address = config
        .get("server_address")
        .map(String::from)
        .unwrap_or("127.0.0.1".to_string());

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

    process(&server_address, mail, use_tls).await.unwrap();
}
