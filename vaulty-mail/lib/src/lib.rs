use chrono::offset::Utc;

pub mod config;
pub mod db;
pub mod storage;
pub mod email;
pub mod mailgun;

mod errors;
pub use errors::Error;

use storage::Backend;

pub struct EmailHandler<'a> {
    date: String,
    storage_token: &'a str,
    storage_backend: storage::Backend,
    storage_path: &'a str,
}

impl<'a> EmailHandler<'a> {
    pub fn new(token: &'a str, backend: &'a str, path: &'a str) -> Self {
        Self {
            storage_token: token,
            storage_backend: backend.into(),
            storage_path: path,

            // TODO: Figure out user's date from email
            // Will be used for naming scrapbook entries
            date: Utc::today().format("%F").to_string(),
        }
    }

    pub async fn handle(&self, email: &email::Email,
                        attachment: Option<email::Attachment>)
        -> Result<(), Error> {
        log::info!("Handling mail for {} on {}",
                   email.recipients[0], self.storage_backend);
        log::info!("Date in UTC: {}", self.date);

        // 1. Figure out if user is valid and active
        // TODO: PGSQL lookup

        // 2. Get user's token and storage location
        // NOTE: Assume the path exists

        // 3. Check what user has configured
        // - Attachments only vs. email content
        // - Create a folder for each day
        // etc.

        // 4. Write all attachments to folder via Dropbox API
        if let Some(attachment) = attachment {
            let attachment = attachment.data();
            let _size = attachment.size;

            let file_path = format!("{}/{}", self.storage_path, attachment.name);

            match self.storage_backend {
                Backend::Dropbox => {
                    // Build a Dropbox client
                    let client = storage::dropbox::Client::from_token(self.storage_token);

                    let result = client.upload(&file_path, attachment.data).await;

                    result.map_err(|e| e.into())
                },
                Backend::Gdrive => {
                    // TODO
                    Ok(())
                },
                Backend::S3 => {
                    // TODO
                    Ok(())
                }
            }
        } else {
            // Just dump the email (scrapbook mode!)
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
}
