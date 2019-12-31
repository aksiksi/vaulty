use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct Mail {
    sender: String,
    recipient: String,
    subject: String,
    #[serde(rename = "body-html")]
    body: String,
    attachments: Option<Vec<Attachment>>
}

#[derive(Deserialize, Debug)]
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

// impl Mail {
//     fn get_attachments(&self) -> &Vec<Attachment> {
//         for &attachment in self.attachments {
//             if let Some(attachment.url) = url {
//                 // Fetch remote content and fill it in
//             }
//         }

//         &self.attachments
//     }
// }
