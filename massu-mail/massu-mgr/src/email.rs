use std::default::Default;

use serde::{Deserialize};

#[derive(Deserialize, Debug, Default)]
pub struct Email {
    sender: String,
    recipient: String,
    subject: String,
    #[serde(rename = "body-html")]
    body: String,
    attachments: Option<Vec<Attachment>>
}

#[derive(Deserialize, Debug, Default)]
pub struct Attachment {
    // Attachment can either contain the full content,
    // or a URL that points to the content
    content: Option<Vec<u8>>,
    url: Option<String>,
    #[serde(rename = "content-type")]
    content_type: String,
    name: String,
    size: usize,
}

/// Represents a single email
impl Email {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_body(body: &str, content_type: &str) ->
        Result<Self, Box<dyn std::error::Error>> {
        if content_type == "application/x-www-form-urlencoded" {
            let mut mail = Self::new();

            let parsed = url::form_urlencoded::parse(body.as_bytes())
                                              .into_owned();

            for (k, v) in parsed {
                if k == "sender" {
                    mail.sender = v;
                } else if k == "recipient" {
                    mail.recipient = v;
                } else if k == "subject" {
                    mail.subject = v;
                } else if k == "body-html" {
                    mail.body = v;
                } else if k == "attachments" {
                    mail.attachments = Some(Attachment::from_raw_json(&v)?);
                }
            }

            Ok(mail)
        } else if content_type == "application/json" {
            match serde_json::from_str::<Self>(body) {
                Ok(m) => Ok(m),
                Err(e) => Err(e.into())
            }
        } else {
            Err(format!("Unknown content type: {}", content_type).into())
        }
    }
}

/// Represents a single email attachment
impl Attachment {
    /// Converts attachments from `[{"url": ..., }]` to a struct.
    pub fn from_raw_json(attachments: &str)
        -> Result<Vec<Attachment>, serde_json::Error> {
        serde_json::from_str(attachments)
    }
}
