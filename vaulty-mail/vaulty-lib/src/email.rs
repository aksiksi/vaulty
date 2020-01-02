pub trait Email {
    type Attachment;

    fn get_recipient(&self) -> &str;
    fn get_sender(&self) -> &str;
    fn get_subject(&self) -> &str;
    fn get_body(&self) -> &str;
    fn get_attachments(&self) -> &Vec<Self::Attachment>;
}

/// Represents a single email attachment
pub trait Attachment {
    fn get_content(&self) -> &Vec<u8>;
    fn get_content_type(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_size(&self) -> usize { self.get_content().len() }
}
