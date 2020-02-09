use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};

use futures::stream::{self, TryStreamExt};
use structopt::StructOpt;

// TODO: Migrate to file or DB lookup in `basic_auth`
const VAULTY_USER: &str = "admin";
const VAULTY_PASS: &str = "test123";

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

async fn send_attachment(attachment: &vaulty::email::Attachment)
    -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Processing attachment for email: {}",
               attachment.get_email_id().to_string());

    let raw = rmp_serde::encode::to_vec_named(&attachment)?;

    let client = reqwest::Client::new();
    let req = client
        .post("http://127.0.0.1:7777/postfix/attachment")
        .header(reqwest::header::CONTENT_TYPE, attachment.get_mime())
        .basic_auth(VAULTY_USER, Some(VAULTY_PASS))
        .body(reqwest::Body::from(raw));

    let resp = req.send().await?;
    let bytes = &resp.bytes().await?;

    let resp_str = std::str::from_utf8(bytes)?;

    log::info!("{}", resp_str);

    Ok(())
}

/// Transmit this email to the Vaulty processing server
async fn process(mail: vaulty::email::Email, raw_mail: &[u8],
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
                stdin.write_all(raw_mail).expect("Failed to write to stdin");
            }

            return Ok(());
        }
    }

    let client = reqwest::Client::new();
    let email = serde_json::to_string(&mail)?;

    let req = client
        .post("http://127.0.0.1:7777/postfix/email")
        .basic_auth(VAULTY_USER, Some(VAULTY_PASS))
        .body(reqwest::Body::from(email));

    let resp = req.send().await?;

    assert!(resp.status().is_success());

    if let Some(attachments) = &mail.attachments {
        // 1. Create a TryStream from an iterator
        // 2. Iterator must be of type `Result`
        // 3. Run a function for each element in the stream, concurrently
        stream::iter(attachments.iter().map(Ok))
               .try_for_each_concurrent(None, send_attachment).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() {
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

    process(mail, email_content.as_bytes(), opt.original_recipients)
        .await
        .unwrap();
}
