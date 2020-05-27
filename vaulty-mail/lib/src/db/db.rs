use crate::email::Email;

use chrono::{DateTime, Utc};
use sqlx::Row;

use crate::storage;
use crate::Error;

pub enum LogLevel {
    Debug,
    Info,
    Warning,
    Error,
}

impl From<i32> for LogLevel {
    fn from(l: i32) -> Self {
        match l {
            0 => LogLevel::Debug,
            1 => LogLevel::Info,
            2 => LogLevel::Warning,
            3 => LogLevel::Error,
            _ => LogLevel::Info,
        }
    }
}

#[allow(dead_code)]
const USER_TABLE: &str = "vaulty_users";
const ADDRESS_TABLE: &str = "vaulty_addresses";
const MAIL_TABLE: &str = "vaulty_mail";
const LOG_TABLE: &str = "vaulty_logs";

/// Single address row in DB
#[derive(Clone)]
pub struct Address {
    pub address: String,
    pub user_id: i32,
    pub email_quota: i32,
    pub num_received: i32,
    pub max_email_size: i32,
    pub storage_quota: i64,
    pub storage_used: i64,
    pub storage_token: String,
    pub storage_backend: storage::Backend,
    pub storage_path: String,
    pub last_renewal_time: DateTime<Utc>,
}

impl Address {
    const TABLE_NAME: &'static str = ADDRESS_TABLE;

    /// Validates sender address by checking that it is in the list of
    /// whitelisted senders for this recipient.
    pub async fn validate_sender(
        &self,
        email: &Email,
        db_client: &mut Client<'_>,
    ) -> Result<bool, Error> {
        let sender = &email.sender;
        let recipient = &self.address;

        let query = format!(
            "SELECT is_active FROM {} WHERE ($1 = ANY (whitelist) OR is_whitelist_enabled = false)
            AND address = $2",
            Self::TABLE_NAME
        );

        let row = sqlx::query(&query)
            .bind(sender)
            .bind(recipient)
            .fetch_optional(db_client.db)
            .await?;

        if row.is_none() {
            let msg = format!(
                "Rejecting email {} (Message-ID: {}): sender {} is not on {} whitelist",
                &email.uuid,
                &email.message_id.as_ref().unwrap_or(&"N/A".to_string()),
                sender,
                recipient
            );
            log::warn!("{}", msg);

            // Do not log this against email as email might not have
            // been inserted yet
            db_client.log(&msg, None, LogLevel::Warning).await;

            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// Update address storage use for this address
    pub async fn update_storage_used(
        &self,
        size: usize, // in bytes
        email_received: bool,
        db_client: &mut Client<'_>,
    ) -> Result<(), Error> {
        let query = if email_received {
            format!(
                "
            UPDATE {}
            SET storage_used = storage_used + {}, num_received = num_received + 1
            WHERE address = $1",
                Self::TABLE_NAME,
                size as i64,
            )
        } else {
            format!(
                "
            UPDATE {}
            SET storage_used = storage_used + {}
            WHERE address = $1",
                Self::TABLE_NAME,
                size as i64
            )
        };

        let _num_rows = sqlx::query(&query)
            .bind(&self.address)
            .execute(db_client.db)
            .await?;

        Ok(())
    }
}

/// Abstraction over sqlx DB client for Vaulty DB
pub struct Client<'a> {
    pub db: &'a mut sqlx::PgPool,
}

impl<'a> Client<'a> {
    pub fn new(db: &'a mut sqlx::PgPool) -> Self {
        Client { db }
    }

    /// Convert a recipient email to a user ID
    pub async fn get_user_id(&mut self, recipient: &str) -> Result<i32, sqlx::Error> {
        let query = format!("SELECT user_id FROM {} WHERE address = $1", ADDRESS_TABLE);

        let row = sqlx::query(&query)
            .bind(recipient)
            .fetch_one(self.db)
            .await?;

        let user_id: i32 = row.get("user_id");

        Ok(user_id)
    }

    /// Convert a list of recipient emails into address info.
    ///
    /// This function will only return info for the **first** valid recipient
    /// email in the provided list.
    pub async fn get_address(
        &mut self,
        recipients: &Vec<&str>,
    ) -> Result<Option<Address>, sqlx::Error> {
        // Build a SQL list of values to check against
        // NOTE: This may need to be sanitizied
        let address_list = recipients
            .iter()
            .map(|r| format!("'{}'", r))
            .collect::<Vec<String>>()
            .join(", ");

        let query = format!(
            "SELECT * FROM {} WHERE address IN ({})",
            ADDRESS_TABLE, &address_list
        );

        let row = sqlx::query(&query).fetch_optional(self.db).await?;

        if let Some(data) = row {
            let address = Address {
                address: data.get("address"),
                user_id: data.get("user_id"),
                email_quota: data.get("email_quota"),
                num_received: data.get("num_received"),
                max_email_size: data.get("max_email_size"),
                storage_quota: data.get("storage_quota"),
                storage_used: data.get("storage_used"),
                storage_token: data.get("storage_token"),
                storage_backend: data.get::<String, &str>("storage_backend").into(),
                storage_path: data.get("storage_path"),
                last_renewal_time: data.get("last_renewal_time"),
            };

            Ok(Some(address))
        } else {
            // If no rows returned, none of the recipients are valid
            Ok(None)
        }
    }

    /// Log a message to the logs table
    ///
    /// If this fails, we just log an error internally and proceed.
    ///
    /// `mail_id` is optional since we may insert logs before inserting an
    /// email (e.g., rejected email).
    pub async fn log(&mut self, msg: &str, mail_id: Option<&uuid::Uuid>, log_level: LogLevel) {
        let query = format!(
            "
            INSERT INTO {0}
            (mail_id, msg, log_level, creation_time) VALUES
            ($1, $2, $3, $4)",
            LOG_TABLE
        );

        let creation_time: DateTime<Utc> = Utc::now();

        let num_rows = sqlx::query(&query)
            .bind(mail_id)
            .bind(msg)
            .bind(log_level as i32)
            .bind(creation_time)
            .execute(self.db)
            .await;

        if let Err(e) = num_rows {
            log::error!("Failed to log to DB: {}", e.to_string());
        }
    }

    /// Insert an email into DB
    /// Status and error message must be updated later
    pub async fn insert_email(&mut self, email: &Email) -> Result<(), sqlx::Error> {
        let mail_id = &email.uuid;

        // Recipient list will have been filtered down at this point
        let recipient = &email.recipients[0];

        let total_size = email.size;
        let creation_time: DateTime<Utc> = Utc::now();
        let last_update_time: DateTime<Utc> = Utc::now();

        let query = format!("
            INSERT INTO {0} (user_id, address_id, id, num_attachments, total_size, message_id, status, error_msg, last_update_time, creation_time) VALUES
            ((SELECT user_id FROM {1} WHERE address = $1),
             (SELECT id FROM {1} WHERE address = $1), $2, $3, $4, $5, $6, $7, $8, $9)",
            MAIL_TABLE, ADDRESS_TABLE
        );

        let _num_rows = sqlx::query(&query)
            .bind(recipient)
            .bind(mail_id)
            .bind(email.num_attachments as i32)
            .bind(total_size as i32)
            .bind(email.message_id.as_ref())
            .bind(true)
            .bind("")
            .bind(last_update_time)
            .bind(creation_time)
            .execute(self.db)
            .await?;

        Ok(())
    }

    /// Update email status (success or failure)
    /// We do not really care if this operation fails (best-effort)
    pub async fn update_email(&mut self, email: &Email, status: bool, msg: Option<&str>) {
        let mail_id = &email.uuid;

        let query = format!(
            "
            UPDATE {}
            SET status = $1, error_msg = $2
            WHERE mail_id = $3",
            MAIL_TABLE
        );

        let num_rows = sqlx::query(&query)
            .bind(status)
            .bind(msg)
            .bind(mail_id)
            .execute(self.db)
            .await;

        if let Err(e) = num_rows {
            log::error!("Failed to update email: {}", e.to_string());
        }
    }
}
