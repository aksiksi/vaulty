use chrono::offset::Utc;

pub mod dropbox;
pub mod email;

pub struct EmailHandler {
    dropbox_client: dropbox::Client,
    date: String,
    // TODO: GDrive client, PGSQL client, etc.
}

impl EmailHandler {
    pub fn new() -> Self {
        Self {
            dropbox_client: dropbox::Client::new(),
            // TODO: Figure out user's date from email
            // Will be used for naming scrapbook entries
            date: Utc::today().format("%F").to_string(),
        }
    }

    pub fn handle(&mut self, email: email::Email) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Handling mail for {}", email.recipient);
        log::info!("Date in UTC: {}", self.date);

        // 1. Figure out if user is valid and active
        // TODO: PGSQL lookup

        // 2. Get user's token and storage location
        // NOTE: Assume the path exists
        let dropbox_token = std::env::var("DROPBOX_TOKEN").unwrap();
        self.dropbox_client.set_token(&dropbox_token);
        let storage_path = "/vaulty";

        // 3. Check what user has configured
        // - Attachments only vs. email content
        // - Create a folder for each day
        // etc.

        // 4. Write all attachments to folder via Dropbox API
        for attachment in &email.attachments {
            let result = self.dropbox_client.upload(format!("{}/{}", storage_path, attachment.name).as_str(),
                                                    &attachment.data, true);
            if let Err(_) = result {
                log::error!("Failed to upload attachment of size = {}", attachment.size);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
}
