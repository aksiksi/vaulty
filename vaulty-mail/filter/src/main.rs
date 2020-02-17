use std::env;
use std::io::Read;
use std::process::Stdio;

// AsyncRead trait for tokio::io::stdin
use tokio::prelude::*;
use tokio::process::Command;

use futures::stream::{FuturesUnordered, StreamExt};

use structopt::StructOpt;

use lazy_static::lazy_static;

// TODO: Can we make this more flexible?
lazy_static! {
    static ref VAULTY_USER: String = env::var("VAULTY_USER")
                                         .expect("No auth username found!");
    static ref VAULTY_PASS: String = env::var("VAULTY_PASS")
                                         .expect("No auth username found!");
}

const VALID_RECIPIENTS: &[&str] = &[
    "postmaster@vaulty.net",
    "admin@vaulty.net",
    "support@vaulty.net",
];

#[derive(Debug, StructOpt)]
#[structopt(name = "vaulty-filter", about = "Vaulty filter for Postfix incoming mail.")]
struct Opt {
    #[structopt(short, long)]
    sender: String,

    #[structopt(short, long)]
    recipients: Vec<String>,

    #[structopt(short, long)]
    original_recipients: Vec<String>,
}

async fn send_attachment(remote_addr: &str, client: &reqwest::Client,
                         attachment: &vaulty::email::Attachment)
    -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Processing attachment for email: {}",
               attachment.get_email_id().to_string());

    let raw = rmp_serde::encode::to_vec_named(&attachment)?;

    let req = client
        .post(&format!("http://{}:7777/postfix/attachment", remote_addr))
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
async fn process(remote_addr: &str, mail: vaulty::email::Email, raw_mail: &[u8],
                 original_recipients: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Any mail destined for "postmaster" (or equivalent) must be injected
    // back into Postfix. The recipient would have already been remapped using
    // the virtual alias mapping, which is why we check the orig. recipient list.
    for r in original_recipients.iter() {
        if VALID_RECIPIENTS.iter().any(|e| e == r) {
            let mut child =
                Command::new("/usr/sbin/sendmail")
                        .args(&["-G", "-i", "-f", &mail.sender, &mail.recipients.join(" ")])
                        .stdin(Stdio::piped())
                        .spawn()?;

            {
                let stdin = child.stdin.as_mut().expect("Failed to open stdin");
                stdin.write_all(raw_mail).await.expect("Failed to write to stdin");
            }

            return Ok(());
        }
    }

    let client = reqwest::Client::new();
    let email = serde_json::to_string(&mail)?;

    let req = client
        .post(&format!("http://{}:7777/postfix/email", remote_addr))
        .basic_auth(VAULTY_USER.as_str(), Some(VAULTY_PASS.as_str()))
        .body(reqwest::Body::from(email));

    let resp = req.send().await?;
    let status = resp.status().is_success();

    let bytes = &resp.bytes().await?;
    let resp_str = std::str::from_utf8(bytes)?;

    if !status {
        log::error!("Failed to process email {} with: \"{}\"",
                    mail.uuid, resp_str);
        return Err("Failed to process email".into());
    }

    if let Some(attachments) = &mail.attachments {
        // 1. Create an iterator of `Future<Result<_, _>>`
        // 2. Collect the futures in a `FuturesUnordered`
        // 3. Collect the results into a `Vec`
        attachments.iter()
                   .map(|a| send_attachment(&remote_addr, &client, a))
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
    std::io::stdin().read_to_string(&mut email_content)
                    .expect("Failed to read email body from stdin!");

    // Parse and process email
    let mail = vaulty::email::Email::from_mime(email_content.as_bytes())
                                    .unwrap()
                                    .with_sender(opt.sender)
                                    .with_recipients(opt.recipients);

    process(&remote_addr, mail, email_content.as_bytes(), opt.original_recipients)
        .await
        .unwrap();
}
