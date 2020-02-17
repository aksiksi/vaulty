use chrono::offset::Utc;

pub mod config;
pub mod db;
pub mod dropbox;
pub mod email;
pub mod errors;
pub mod mailgun;

use errors::VaultyError;

pub struct EmailHandler<'a> {
    date: String,
    storage_token: &'a str,
    storage_backend: &'a str,
    storage_path: &'a str,
}

impl<'a> EmailHandler<'a> {
    pub fn new(token: &'a str, backend: &'a str, path: &'a str) -> Self {
        Self {
            storage_token: token,
            storage_backend: backend,
            storage_path: path,

            // TODO: Figure out user's date from email
            // Will be used for naming scrapbook entries
            date: Utc::today().format("%F").to_string(),
        }
    }

    pub async fn handle(&self, email: &email::Email,
                        attachment: Option<email::Attachment>)
        -> Result<(), VaultyError> {
        log::info!("Handling mail for {} on {}",
                   email.recipients[0], self.storage_backend);
        log::info!("Date in UTC: {}", self.date);

        // 1. Figure out if user is valid and active
        // TODO: PGSQL lookup

        // 2. Get user's token and storage location
        // NOTE: Assume the path exists
        let client = dropbox::Client::from_token(self.storage_token);

        // 3. Check what user has configured
        // - Attachments only vs. email content
        // - Create a folder for each day
        // etc.

        // 4. Write all attachments to folder via Dropbox API
        if let Some(attachment) = attachment {
            let attachment = attachment.data();
            let size = attachment.size;

            let file_path = format!("{}/{}", self.storage_path, attachment.name);
            let result = client.upload(&file_path, attachment.data, true).await;

            result.map(|_| ())
                  .map_err(|e| {
                      log::error!("Failed to upload attachment of size = {}",
                                  size);
                      VaultyError { msg: e.to_string() }
                  })
        } else {
            // Just dump the email (scrapbook mode!)
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
}
