use lettre::{smtp::extension::ClientId, SendableEmail, SmtpClient, Transport};
use lettre_email::Email;

use vaulty::api::ServerResult;

use crate::error::Error;

pub fn reply(mail: &vaulty::email::Email, body: String) {
    if mail.message_id.is_none() {
        // We cannot reply to a message with no Message-ID!
        log::error!("Mail has no Message-ID!");
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

/// Send out a reply to the user containing a description of why their email
/// was not processed correctly.
pub fn reply_error(err: Error) -> i32 {
    // SMTP status code
    let status_code = match &err {
        Error::Temporary => {
            // If this was unexpected server error (e.g., timeout), tell Postfix
            // to retry delivery of this email to the filter.
            // Do not inform the user about this.
            // TODO: Record a metric for such failures to help debug issues when
            // they arise.
            return super::TEMPFAIL;
        }
        Error::Unexpected => Some("5.5.4"),
        // See: https://www.iana.org/assignments/smtp-enhanced-status-codes/smtp-enhanced-status-codes.xhtml
        Error::Server(result) => match &result.error {
            Some(err) => match err {
                vaulty::Error::InvalidRecipient => Some("5.1.1"),
                vaulty::Error::QuotaExceeded(_) => Some("5.2.3"),
                vaulty::Error::SenderNotWhitelisted { .. } => Some("5.7.1"),
                vaulty::Error::TokenExpired | vaulty::Error::Unauthorized => Some("5.7.8"),
                _ => Some("5.2.0"),
            },
            None => None,
        },
    };

    if let Some(code) = status_code {
        println!("{}: {}", code, err.to_string());
        super::UNAVAILABLE
    } else {
        // If we're here, this email was successful?
        log::warn!("Successful email, but marked as failed?");
        0
    }
}

pub fn reply_success(mail: &vaulty::email::Email, result: ServerResult) -> i32 {
    let body = format!(
        "Vaulty successfully uploaded {} attachments to {}!",
        result.num_attachments.unwrap(),
        result.storage_backend.unwrap()
    );

    reply(mail, body);

    return 0;
}
