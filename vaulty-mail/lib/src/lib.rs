use bytes::Bytes;
use chrono::offset::Utc;
use futures::stream::Stream;

pub mod config;
pub mod constants;
pub mod db;
pub mod email;
pub mod mailgun;
pub mod storage;

mod error;
pub use error::Error;

use storage::client::Client;
use storage::dropbox::client::DropboxClient;
use storage::Backend;

pub struct EmailHandler<'a> {
    date: String,
    storage_token: &'a str,
    storage_backend: &'a storage::Backend,
    storage_path: &'a str,
}

impl<'a> EmailHandler<'a> {
    pub fn new(token: &'a str, backend: &'a storage::Backend, path: &'a str) -> Self {
        Self {
            storage_token: token,
            storage_backend: backend,
            storage_path: path,

            // TODO: Figure out user's date from email
            // Will be used for naming scrapbook entries
            date: Utc::today().format("%F").to_string(),
        }
    }

    pub async fn handle(
        &self,
        email: &email::Email,
        attachment: Option<impl Stream<Item = Result<Bytes, Error>> + Send + Sync + 'static>,
        attachment_name: String,
        _attachment_size: usize,
    ) -> Result<(), Error> {
        log::info!(
            "Handling mail for {} on {}",
            email.recipients[0],
            self.storage_backend
        );
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
            let file_path = format!("{}/{}", self.storage_path, attachment_name);

            match self.storage_backend {
                Backend::Dropbox => {
                    // Build a Dropbox client
                    let client = DropboxClient::from_token(self.storage_token);
                    let result = client.upload_stream(&file_path, attachment).await;

                    result.map_err(|e| e.into())
                }
                Backend::Gdrive => {
                    // TODO
                    Ok(())
                }
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
mod tests {}
