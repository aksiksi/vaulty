use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};

use structopt::StructOpt;

static VALID_RECIPIENTS: &[&str] = &[
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

/// Transmit this email to the Vaulty processing server
fn process(mail: vaulty::email::Email, raw_mail: &[u8],
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

    let client = reqwest::blocking::Client::new();
    let json = serde_json::to_string(&mail)?;

    let req = client
        .post("127.0.0.1:7777/filter")
        .header("VAULTY_SENDER", &mail.sender)
        .header("VAULTY_RECIPIENTS", &mail.recipients.join(","))
        .body(reqwest::blocking::Body::from(json));

    let resp = req.send()?;

    assert!(resp.status().is_success());

    let body = resp.text()?;

    println!("{}", body);
    log::info!("{}", body);

    Ok(())
}

fn main() {
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

    process(mail, email_content.as_bytes(), opt.original_recipients).unwrap();
}
