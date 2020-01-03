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
            date: Utc::today().format("%F").to_string(),
        }
    }

    pub fn handle(&mut self, _email: email::Email) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Figure out if user is valid and active
        // TODO: PGSQL lookup

        // 2. Get user's token and storage location
        // NOTE: Assume the path exists
        let dropbox_token = std::env::var("DROPBOX_TOKEN").unwrap();
        self.dropbox_client.set_token(&dropbox_token);
        let _storage_path = "/abcd";

        // 3. Check what user has configured
        // - Attachments only vs. email content
        // - Create a folder for each day
        // etc.
        println!("Date: {}", self.date);

        // 4. Create day folder, if applicable
        Ok(())
    }
}

#[cfg(test)]
mod tests {
}
