use lettre::{smtp::extension::ClientId, SendableEmail, SmtpClient, Transport};
use lettre_email::Email;

#[derive(Debug)]
pub enum Error {
    Server(String),
    Unexpected,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            Error::Server(ref msg) => write!(f, "{}", msg),
            Error::Unexpected => write!(
                f,
                "An unexpected error occurred while processing this email.\n\n
                 Please contact Vaulty support: https://groups.google.com/forum/#!forum/vaulty-support"
            ),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(_err: reqwest::Error) -> Self {
        Self::Unexpected
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(_err: serde_json::error::Error) -> Self {
        Self::Unexpected
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_err: std::str::Utf8Error) -> Self {
        Self::Unexpected
    }
}

/// Send out a reply to the user containing a description of why their email
/// was not processed correctly.
pub fn reply_with_error(mail: &vaulty::email::Email, err: Error) {
    let body = format!(
        "Vaulty encountered an error while processing this email:\n\n{}",
        err.to_string()
    );

    if mail.message_id.is_none() {
        // We cannot reply to a message with no Message-ID!
        return;
    }

    // Build a Message-ID surrounded by <>
    let message_id = mail
        .message_id
        .as_ref()
        .map(|s| format!("<{}>", s))
        .unwrap();

    // Gmail apparently uses the Subject + In-Reply-To to thread emails
    let subject = mail
        .subject
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("Mail processing failed")
        .to_string();

    let email: SendableEmail = Email::builder()
        .to(mail.sender.clone())
        .from("noreply@vaulty.net")
        .subject(format!("Re: {}", subject))
        .in_reply_to(message_id.clone())
        .references(message_id.clone())
        // TODO: Add `message_id` call once Lettre creates a new release
        .text(body)
        .build()
        .unwrap()
        .into();

    // Open a local connection on port 25
    // NOTE: Must be changed if server is moved to another box
    let mut mailer = SmtpClient::new_unencrypted_localhost()
        .unwrap()
        // NOTE: You can change this to any FQDN
        .hello_name(ClientId::hostname())
        .transport();

    // Send the email
    let result = mailer.send(email);

    if result.is_ok() {
        log::debug!("Email sent");
    } else {
        log::error!("Could not send email: {:?}", result);
    }
}
