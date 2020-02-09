use chrono::offset::Utc;

pub mod dropbox;
pub mod email;
pub mod mailgun;

pub struct EmailHandler {
    date: String,
    // TODO: GDrive client, PGSQL client, etc.
}

impl EmailHandler {
    pub fn new() -> Self {
        Self {
            // TODO: Figure out user's date from email
            // Will be used for naming scrapbook entries
            date: Utc::today().format("%F").to_string(),
        }
    }

    pub async fn handle(&self, email: &email::Email, attachment: Option<email::Attachment>)
        -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Handling mail for {}", email.recipients[0]);
        log::info!("Date in UTC: {}", self.date);

        // 1. Figure out if user is valid and active
        // TODO: PGSQL lookup

        // 2. Get user's token and storage location
        // NOTE: Assume the path exists
        let dropbox_token = std::env::var("DROPBOX_TOKEN").unwrap();
        let dropbox_client = dropbox::Client::from_token(dropbox_token);
        let storage_path = "/vaulty";

        // 3. Check what user has configured
        // - Attachments only vs. email content
        // - Create a folder for each day
        // etc.

        // 4. Write all attachments to folder via Dropbox API
        if let Some(attachment) = attachment {
            let attachment = attachment.data();

            let file_path = format!("{}/{}", storage_path, attachment.name);
            let result = dropbox_client.upload(&file_path, attachment.data, true).await;

            if result.is_err() {
                log::error!("Failed to upload attachment of size = {}", attachment.size);
            }

            // Throw away the result
            result.map(|_| ())
        } else {
            // Just dump the email (scrapbook mode!)
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
}