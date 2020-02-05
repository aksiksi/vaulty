use std::io::Read;
use std::io::Write;
use std::process::{Command, Stdio};

use structopt::StructOpt;

mod email;

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
fn process(mail: email::Email, raw_mail: &[u8],
           sender: String, recipients: Vec<String>,
           original_recipients: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    // Any mail destined for "postmaster" (or equivalent) must be injected
    // back into Postfix. The recipient would have already been remapped using
    // the virtual alias mapping, which is why we check the orig. recipient list.
    for r in original_recipients.iter() {
        if VALID_RECIPIENTS.iter().any(|e| e == r) {
            let mut child =
                Command::new("/usr/sbin/sendmail")
                        .args(&["-G", "-i", "-f", &sender, &recipients.join(" ")])
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

    let req = client
        .post("http://httpbin.org/post")
        .header("VAULTY_SENDER", sender)
        .header("VAULTY_RECIPIENTS", recipients.join(","))
        .body(reqwest::blocking::Body::from(mail.body));

    let resp = req.send()?;

    assert!(resp.status().is_success());

    let body = resp.text()?;

    println!("{}", body);

    Ok(())
}

fn main() {
    // Init logger
    env_logger::builder().format_timestamp_micros().init();

    let opt = Opt::from_args();

    println!("{:?}", opt);

    // Get message body from stdin
    let mut email_content = String::new();
    std::io::stdin().read_to_string(&mut email_content)
                    .expect("Failed to read email body from stdin!");

    // Parse and process email
    let mail = email::Email::from_mime(email_content.as_bytes()).unwrap();

    process(mail, email_content.as_bytes(), opt.sender,
            opt.recipients, opt.original_recipients).unwrap();
}
