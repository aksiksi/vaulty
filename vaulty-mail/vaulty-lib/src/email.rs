/// Generic Email and Attachment implementations.
/// The idea is to use service-specific types for interacting
/// with APIs, and then implement `Into` these types.
pub struct Email {
    pub sender: String,
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<Attachment>,
}

pub struct Attachment {
    pub data: Vec<u8>,
    pub content_type: String,
    pub name: String,
    pub size: usize,
}
