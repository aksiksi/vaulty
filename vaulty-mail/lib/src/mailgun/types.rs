use std::collections::HashMap;
use std::convert::From;
use std::default::Default;

use serde::Deserialize;

// TODO: Move this out into a trait and implement a
// basic version for MG, SES, and Postfix (?)
#[derive(Deserialize, Debug, Default)]
pub struct Email {
    sender: String,
    recipient: String,
    subject: String,
    #[serde(rename = "body-plain")]
    body: String,
    #[serde(rename = "body-html")]
    body_html: String,
}

#[derive(Deserialize, Debug, Default)]
struct AttachmentJson {
    attachments: Vec<Attachment>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Attachment {
    // Attachment can either contain the full content,
    // or a URL that points to the content
    pub content: Option<Vec<u8>>,
    pub url: String,
    #[serde(rename = "content-type")]
    content_type: String,
    pub name: String,
    size: usize,
}

/// Represents a single email as provided by Mailgun
impl Email {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn from_form(body: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut mail = Self::new();
        let parsed = url::form_urlencoded::parse(body.as_bytes()).into_owned();

        for (k, v) in parsed {
            if k == "sender" {
                mail.sender = v;
            } else if k == "recipient" {
                mail.recipient = v;
            } else if k == "subject" {
                mail.subject = v;
            } else if k == "body-plain" {
                mail.body = v;
            } else if k == "body-html" {
                mail.body_html = v;
            }
        }

        Ok(mail)
    }

    pub fn from_json(body: &str) -> Result<Self, Box<dyn std::error::Error>> {
        serde_json::from_str::<Self>(body).map_err(|e| e.into())
    }
}

impl From<Email> for crate::email::Email {
    fn from(email: Email) -> crate::email::Email {
        crate::email::Email {
            sender: email.sender,
            recipient: email.recipient,
            subject: email.subject,
            body: email.body,
        }
    }
}

/// Represents a single email attachment
impl Attachment {
    /// Create a Vec of attachments from a Mailgun form response
    pub fn from_form(body: &str) -> Result<Vec<Attachment>, Box<dyn std::error::Error>> {
        let parsed: HashMap<String, String> =
            url::form_urlencoded::parse(body.as_bytes()).into_owned().collect();
        let attachments_str = parsed.get("attachments").unwrap();

        serde_json::from_str::<Vec<Attachment>>(attachments_str).map_err(|e| e.into())
    }

    /// Create a Vec of attachments from a Mailgun JSON response
    pub fn from_json(body: &str) -> Result<Vec<Attachment>, Box<dyn std::error::Error>> {
        serde_json::from_str::<AttachmentJson>(body)
            .map_err(|e| e.into())
            .map(|json| json.attachments)
    }

    /// If the attachment has a URL but no content, grab the attachment
    /// content. Data is filled into the current struct.
    pub async fn fetch(mut self) -> Result<Self, Box<dyn std::error::Error>> {
        if self.content.is_some() {
            return Ok(self);
        }

        let api_key = std::env::var("MAILGUN_API_KEY");
        let client = reqwest::Client::new();

        let resp = client
            .get(reqwest::Url::parse(&self.url)?)
            .basic_auth("api", api_key.ok())
            .send()
            .await?
            .error_for_status()?;

        let buf = &resp.bytes().await?;

        self.content = Some(buf.to_vec());

        Ok(self)
    }
}

impl From<Attachment> for crate::email::Attachment {
    fn from(attachment: Attachment) -> crate::email::Attachment {
        crate::email::Attachment {
            data: attachment.content.unwrap(),
            content_type: attachment.content_type,
            name: attachment.name,
            size: attachment.size,
        }
    }
}
