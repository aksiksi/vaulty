use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Email {
    /// Plaintext body
    pub body: String,

    /// HTML body, if any
    pub body_html: Option<String>,

    /// List of attachments, if any
    pub attachments: Vec<Attachment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AttachmentType {
    Inline,
    Regular,
}

impl Default for AttachmentType {
    fn default() -> Self {
        AttachmentType::Regular
    }
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Attachment {
    /// Attachment type (regular or inline)
    #[serde(rename = "type")]
    pub type_: AttachmentType,

    /// MIME type of attachment (e.g., text/plain)
    pub mime: String,

    /// Charset used to encode the attachment data.
    /// This may be required at the decode side.
    pub charset: String,

    /// Content-ID is set for *inline* attachments.
    /// This ID is used to map the attachment to the image in HTML.
    /// For example: <img src="cid:abcd">
    pub content_id: Option<String>,

    /// Attachment filename
    pub name: String,

    /// Attachment size, in bytes
    pub size: usize,

    /// Attachment data, encoded using charset
    pub data: Vec<u8>,
}

impl Email {
    pub fn new() -> Email {
        Default::default()
    }

    /// Recursively walk the MIME parts and extract the following:
    ///
    /// 1. Body (text and/or html)
    /// 2. Inline attachments
    /// 3. Regular attachments
    ///
    fn parse_recursive(&mut self, part: &mailparse::ParsedMail) -> Result<(), Box<dyn std::error::Error>> {
        let content_type = &part.ctype;
        let mimetype = &content_type.mimetype;

        // If this is an attachment, append to Vec and return
        if let Some(attachment) = Attachment::from_mime(part) {
            self.attachments.push(attachment);
            return Ok(());
        }

        // Email body
        if mimetype.starts_with("text/") {
            let body = part.get_body()?;

            if mimetype.ends_with("plain") {
                self.body = body;
            } else if mimetype.ends_with("html") {
                self.body_html = Some(body);
            }

            return Ok(());
        }

        // Multipart -> process each subpart recursively
        if mimetype.starts_with("multipart/") {
            for subpart in part.subparts.iter() {
                match self.parse_recursive(subpart) {
                    Ok(_) => (),
                    Err(_) => (),
                };
            }
        }

        return Ok(());
    }

    /// Convert a raw MIME email into structured format
    pub fn from_mime(mime_content: &[u8]) -> Result<Email, Box<dyn std::error::Error>> {
        let parsed = mailparse::parse_mail(mime_content)?;

        let mut email = Email::new();
        email.parse_recursive(&parsed)?;

        Ok(email)
    }
}

impl Attachment {
    pub fn new() -> Attachment {
        Default::default()
    }

    /// Inspect part headers to determine if this is an attachment.
    /// If it is, build the Attachment and returns it.
    fn from_mime(part: &mailparse::ParsedMail) -> Option<Attachment> {
        let content_type = &part.ctype;
        let mimetype = &content_type.mimetype;
        let charset = &content_type.charset.to_lowercase();

        let mut content_disposition = None;
        let mut content_id = None;

        for header in part.headers.iter() {
            let key = header.get_key().unwrap();
            let val = header.get_value().unwrap();

            if key == "Content-Disposition" {
                content_disposition = Some(val.split(";").next()?.to_string());
            } else if key == "Content-ID" {
                // NOTE: actually <cid>
                // angle brackets need to be cleaned up
                content_id = Some(val);
            }
        }

        if content_disposition.is_none() {
            // Not an attachment
            return None;
        }

        // If the content disposition is inline AND MIME is text,
        // likely not an attachment...
        let kind = content_disposition.unwrap();
        if kind == "inline" && mimetype.starts_with("text/") {
            return None;
        }

        let mut attachment = Attachment::new();

        // Build attachment struct
        attachment.mime = mimetype.to_string();
        attachment.charset = charset.to_string();
        attachment.name = content_type.params["name"].clone();
        attachment.data = match part.get_body_raw() {
            Ok(body) => body,
            Err(_) => {
                log::error!("Attachment body not found");
                return None;
            },
        };

        attachment.size = attachment.data.len();
        attachment.content_id = content_id;

        if kind == "attachment" {
            attachment.type_ = AttachmentType::Regular;
        } else if kind == "inline" {
            attachment.type_ = AttachmentType::Inline;
        } else {
            log::error!("Invalid Content-Disposition type: {}", kind);
            return None;
        }

        Some(attachment)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    static SAMPLE_EMAIL_PATHS: &[&str] = &[
        // Content (multipart/alternative), Attachment, Attachment
        concat!(env!("CARGO_MANIFEST_DIR"), "/resources", "/sample_email_1.txt"),

        // Content + Inline Attachment, Attachment, Attachment
        concat!(env!("CARGO_MANIFEST_DIR"), "/resources", "/sample_email_2.txt"),
    ];

    fn get_mail(path: &str) -> Email {
        let mut mail_file = File::open(path).unwrap();
        let mut mail_content = String::new();
        mail_file.read_to_string(&mut mail_content).unwrap();

        Email::from_mime(mail_content.as_bytes()).unwrap()
    }

    #[test]
    fn parse_body() {
        let mail_path = SAMPLE_EMAIL_PATHS[0];
        let mail = get_mail(mail_path);

        assert_eq!(mail.body, "AAFAFAF\n\n");
    }

    #[test]
    fn parse_attachments() {
        let mail_path = SAMPLE_EMAIL_PATHS[0];
        let mail = get_mail(mail_path);

        assert_eq!(mail.attachments.len(), 2);
        assert_eq!(mail.attachments[0].name, "hello.cpp");
        assert_eq!(mail.attachments[1].size, 7946);

        assert!(match mail.attachments[1].type_ {
            AttachmentType::Regular => true,
            AttachmentType::Inline => false,
        });
    }

    #[test]
    fn parse_inline_attachments() {
        let mail_path = SAMPLE_EMAIL_PATHS[1];
        let mail = get_mail(mail_path);

        assert_eq!(mail.attachments.len(), 3);
        assert_eq!(mail.attachments[0].name, "logo.gif");
        assert_eq!(mail.attachments[1].size, 3265);

        assert!(match mail.attachments[1].type_ {
            AttachmentType::Regular => false,
            AttachmentType::Inline => true,
        });
    }
}
