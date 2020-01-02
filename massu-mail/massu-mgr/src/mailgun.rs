use std::default::Default;
use std::io::{Read, Write};
use std::fs::OpenOptions;

use reqwest::blocking::{Client, Response};

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
    attachments: Vec<Attachment>
}

#[derive(Deserialize, Debug, Default)]
pub struct Attachment {
    // Attachment can either contain the full content,
    // or a URL that points to the content
    pub content: Option<Vec<u8>>,
    pub url: Option<String>,
    #[serde(rename = "content-type")]
    content_type: String,
    name: String,
    size: usize,
}

/// Represents a single email as provided by Mailgun
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
                } else if k == "body-plain" {
                    mail.body = v;
                } else if k == "body-html" {
                    mail.body_html = v;
                } else if k == "attachments" {
                    mail.attachments = Attachment::from_raw_json(&v)?;
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

    /// Fetch all attachments associated with this email
    /// Attachments are fetched once
    pub fn fetch_attachments(&mut self)
        -> Result<&Vec<Attachment>, Box<dyn std::error::Error>> {
        let mut failed = false;

        for attachment in &mut self.attachments {
            if attachment.fetch().is_err() {
                log::error!("Failed to fetch attachment: {}",
                            attachment.url.as_ref().unwrap());
                failed = true;
            }
        }

        if failed {
            Err("One or more attachments failed!".into())
        } else {
            Ok(&self.attachments)
        }
    }
}

/// Represents a single email attachment
impl Attachment {
    /// Creates a Vec of attachments from `[{"url": ..., }]`
    pub fn from_raw_json(attachments: &str)
        -> Result<Vec<Attachment>, serde_json::Error> {
        serde_json::from_str(attachments)
    }

    /// If the attachment has a URL but no content, grab the attachment
    /// content. Data is filled into the current struct.
    pub fn fetch(&mut self)
        -> Result<(), Box<dyn std::error::Error>> {
        if self.content.is_some() {
            return Ok(());
        }

        let api_key = std::env::var("MAILGUN_API_KEY");
        let client = Client::new();

        let url = self.url.as_ref().unwrap();
        let mut resp = client.get(reqwest::Url::parse(url)?)
                             .basic_auth("api", api_key.ok())
                             .send()?
                             .error_for_status()?;

        let mut buf: Vec<u8> = Vec::new();
        resp.read_to_end(&mut buf)?;

        self.content = Some(buf);

        Ok(())
    }
}
