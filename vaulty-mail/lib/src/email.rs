use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Unique UUID namespace (URL + vaulty.net)
const UUID_NAMESPACE: &str = "11d00b11-d9d0-5831-a6f7-8f88f86f870a";

/// Represents a single parsed MIME email.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Email {
    /// Email metadata
    pub sender: String,
    pub recipients: Vec<String>,
    pub subject: Option<String>,

    /// Plaintext body
    pub body: String,

    /// HTML body, if any
    pub body_html: Option<String>,

    /// Total email size, in bytes
    pub size: usize,

    /// Number of attachments, if any
    pub num_attachments: u16,

    /// List of attachments, if any
    ///
    /// This is used to keep track of attachments, but is *not* serialized.
    /// Instead, we use the `email_id` field to tie each attachment to an
    /// email on the server.
    #[serde(skip)]
    pub attachments: Option<Vec<Attachment>>,

    /// UUID for this email
    ///
    /// This ties an email to its attachments.
    pub uuid: Uuid,

    /// Message-ID for this email, if found
    pub message_id: Option<String>,
}

/// A single attachment.
///
/// An attachment can either be inline or regular.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Attachment {
    Inline(AttachmentData),
    Regular(AttachmentData),
}

/// Represents the data for an email attachment.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AttachmentData {
    /// MIME type of attachment (e.g., text/plain)
    pub mime: String,

    /// Charset used to encode the attachment data.
    /// This may be required at the decode side.
    pub charset: Option<String>,

    /// Content-ID is used to map the attachment to the image in HTML.
    /// Note that this can be set for regular attachments.
    /// For example: <img src="cid:abcd">
    pub content_id: Option<String>,

    /// Attachment filename
    pub name: String,

    /// Attachment size, in bytes
    pub size: usize,

    /// Attachment data, encoded using charset
    pub data: Vec<u8>,

    /// Index of this attachment in the email
    pub index: u16,

    /// Associated email's UUID
    pub email_id: Uuid,
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
    fn parse_recursive(
        &mut self,
        part: &mailparse::ParsedMail,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content_type = &part.ctype;
        let mimetype = &content_type.mimetype.to_lowercase();

        // If this is an attachment, append to Vec and return
        if let Some(mut attachment) = Attachment::from_mime(part) {
            // Assign email's UUID to this attachment
            attachment.data_mut().email_id = self.uuid;
            attachment.data_mut().index = self.num_attachments;

            self.num_attachments += 1;

            // Add the attachment to the Vec, or construct a new Vec
            if let Some(v) = &mut self.attachments {
                v.push(attachment);
            } else {
                let mut v = Vec::new();
                v.push(attachment);
                self.attachments = Some(v);
            }

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

    /// Extract relevant headers from email
    /// For now, this is limited to Subject and Message-ID
    fn parse_headers(&mut self, part: &mailparse::ParsedMail) {
        // NOTE(aksiksi): Can header names be lowercase?
        let headers = part
            .headers
            .iter()
            .filter(|h| {
                let k = h.get_key().unwrap();
                ["Subject", "Message-ID"].contains(&k.as_str())
            })
            .map(|h| (h.get_key().unwrap(), h.get_value().ok()));

        for (k, v) in headers {
            if k == "Subject" {
                self.subject = v;
            } else if k == "Message-ID" {
                // Extract message ID, if available
                self.message_id = v.map(|s| s.replace("<", "").replace(">", ""));
            }
        }
    }

    /// Generates a deterministic UUID for this email based on metadata.
    /// The idea is that the UUID should be the same for the same email.
    fn generate_uuid(&self) -> Uuid {
        let mut buf = Vec::new();

        if let Some(message_id) = &self.message_id {
            buf.extend(message_id.as_bytes());
        }

        if let Some(subject) = &self.subject {
            buf.extend(subject.as_bytes());
        }

        buf.extend(self.sender.as_bytes());

        for r in &self.recipients {
            buf.extend(r.as_bytes());
        }

        let uuid = Uuid::parse_str(UUID_NAMESPACE).unwrap();

        Uuid::new_v5(&uuid, &buf)
    }

    /// Convert a raw MIME email into structured format
    pub fn from_mime(mime_content: &[u8]) -> Result<Email, Box<dyn std::error::Error>> {
        let parsed = mailparse::parse_mail(mime_content)?;

        let mut email = Email::new();

        // Size of email, in bytes
        email.size = mime_content.len();

        // Parse mail headers
        // This will overwrite the UUID above if "Message-ID" is found
        email.parse_headers(&parsed);

        // Parse body and attachments
        email.parse_recursive(&parsed)?;

        // Assign a deterministic UUID to this email
        email.uuid = email.generate_uuid();

        Ok(email)
    }

    pub fn with_sender(self, sender: String) -> Self {
        Self { sender, ..self }
    }

    pub fn with_recipients(self, recipients: Vec<String>) -> Self {
        Self { recipients, ..self }
    }
}

impl From<&[u8]> for Email {
    fn from(val: &[u8]) -> Self {
        if let Ok(e) = Email::from_mime(val) {
            e
        } else {
            Default::default()
        }
    }
}

impl Attachment {
    /// Inspect part headers to determine if this is an attachment.
    /// If it is, build the Attachment and return it.
    fn from_mime(part: &mailparse::ParsedMail) -> Option<Attachment> {
        let content_type = &part.ctype;
        let mimetype = &content_type.mimetype.to_lowercase();
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

        let mut d = AttachmentData::new();

        // Build attachment struct
        d.mime = mimetype.to_string();
        d.charset = Some(charset.to_string());
        d.name = content_type.params["name"].clone();
        d.data = match part.get_body_raw() {
            Ok(body) => body,
            Err(_) => {
                log::error!("Attachment body not found");
                return None;
            }
        };
        d.size = d.data.len();
        d.content_id = content_id;

        let attachment;

        if kind == "attachment" {
            attachment = Attachment::Regular(d);
        } else if kind == "inline" {
            attachment = Attachment::Inline(d);
        } else {
            log::error!("Invalid Content-Disposition type: {}", kind);
            return None;
        }

        Some(attachment)
    }

    pub fn get_name(&self) -> &String {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => &d.name,
        }
    }

    pub fn get_size(&self) -> usize {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => d.size,
        }
    }

    pub fn get_mime(&self) -> &String {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => &d.mime,
        }
    }

    pub fn get_email_id(&self) -> &uuid::Uuid {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => &d.email_id,
        }
    }

    pub fn get_index(&self) -> u16 {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => d.index,
        }
    }

    pub fn get_data(&self) -> &Vec<u8> {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => &d.data,
        }
    }

    pub fn get_data_owned(self) -> Vec<u8> {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => d.data,
        }
    }

    pub fn is_regular(&self) -> bool {
        match self {
            Attachment::Inline(_) => false,
            Attachment::Regular(_) => true,
        }
    }

    pub fn is_inline(&self) -> bool {
        match self {
            Attachment::Inline(_) => true,
            Attachment::Regular(_) => false,
        }
    }

    /// Get `AttachmentData` struct
    pub fn data(self) -> AttachmentData {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => d,
        }
    }

    /// Get reference to `AttachmentData`
    pub fn data_ref(&self) -> &AttachmentData {
        match self {
            Attachment::Inline(d) | Attachment::Regular(d) => &d,
        }
    }

    /// Get mutable ref to `AttachmentData` struct
    pub fn data_mut(&mut self) -> &mut AttachmentData {
        match self {
            Attachment::Inline(ref mut d) | Attachment::Regular(ref mut d) => d,
        }
    }
}

impl From<&mailparse::ParsedMail<'_>> for Attachment {
    fn from(parsed: &mailparse::ParsedMail<'_>) -> Self {
        if let Some(a) = Attachment::from_mime(parsed) {
            a
        } else {
            Default::default()
        }
    }
}

impl Default for Attachment {
    fn default() -> Self {
        Attachment::Regular(AttachmentData::new())
    }
}

impl AttachmentData {
    fn new() -> AttachmentData {
        Default::default()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs::File;
    use std::io::Read;

    static SAMPLE_EMAIL_PATHS: &[&str] = &[
        // Content (multipart/alternative), Attachment, Attachment
        concat!(env!("CARGO_MANIFEST_DIR"), "/test", "/sample_email_1.txt"),
        // Content + Inline Attachment, Attachment, Attachment
        concat!(env!("CARGO_MANIFEST_DIR"), "/test", "/sample_email_2.txt"),
    ];

    fn get_mail(path: &str) -> Email {
        let mut mail_file = File::open(path).unwrap();
        let mut mail_content = String::new();
        mail_file.read_to_string(&mut mail_content).unwrap();

        Email::from(mail_content.as_bytes())
    }

    #[test]
    fn parse_body() {
        let mail_path = SAMPLE_EMAIL_PATHS[0];
        let mail = get_mail(mail_path);

        assert_eq!(mail.body, "AAFAFAF\n\n");
        assert_eq!(mail.subject.unwrap(), "ABC");

        // Verify the deterministic UUID
        assert_eq!(
            mail.uuid.to_string(),
            "db6377b6-e9c2-5eb0-a4f7-b2ab8bc66042".to_string()
        );
    }

    #[test]
    fn parse_attachments() {
        let mail_path = SAMPLE_EMAIL_PATHS[0];
        let mail = get_mail(mail_path);

        let attachments = &mail.attachments.unwrap();

        assert_eq!(attachments.len(), 2);
        assert_eq!(attachments[0].get_name(), "hello.cpp");
        assert_eq!(attachments[1].get_size(), 7946);

        assert!(attachments[1].is_regular());
    }

    #[test]
    fn parse_inline_attachments() {
        let mail_path = SAMPLE_EMAIL_PATHS[1];
        let mail = get_mail(mail_path);

        let attachments = &mail.attachments.unwrap();

        assert_eq!(attachments.len(), 3);
        assert_eq!(attachments[0].get_name(), "logo.gif");
        assert_eq!(attachments[1].get_size(), 3265);

        assert!(attachments[1].is_inline());
    }
}
