use sqlx::Row;

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

/// Single address row in DB
pub struct Address {
    pub address: String,
    pub user_id: i32,
    pub max_email_size: i32,
    pub quota: i32,
    pub received: i32,
    pub last_renewal_time: chrono::DateTime<chrono::Utc>,
}

/// Abstraction over sqlx DB client for Vaulty DB
pub struct Client<'a> {
    pub db: &'a mut sqlx::PgPool,
    pub user_table: String,
    pub address_table: String,
    pub log_table: String,
}

impl <'a> Client<'a> {
    pub fn new(db: &'a mut sqlx::PgPool) -> Self {
        Client {
            db: db,
            user_table: "users".to_string(),
            address_table: "addresses".to_string(),
            log_table: "logs".to_string(),
        }
    }

    /// Convert a recipient email to a user ID
    pub async fn get_user_id(&mut self, recipient: &str)
        -> Result<i32, Box<dyn std::error::Error>> {
        let query =
            format!("SELECT user_id FROM {} WHERE address = $1",
                    &self.address_table);

        let row = sqlx::query(&query)
                       .bind(recipient)
                       .fetch_one(self.db)
                       .await?;

        let user_id: i32 = row.get("user_id");

        Ok(user_id)
    }

    /// Convert a recipient email to address info
    pub async fn get_address(&mut self, recipient: &str)
        -> Result<Address, Box<dyn std::error::Error>> {
        let query = format!("
            SELECT * FROM {} WHERE address = $1", &self.address_table
        );

        log::info!("{}", query);

        let row = sqlx::query(&query)
                       .bind(recipient)
                       .fetch_optional(self.db)
                       .await?;

        let row = row.unwrap();

        let address = Address {
            address: recipient.to_string(),
            user_id: row.get("user_id"),
            max_email_size: row.get("max_email_size"),
            quota: row.get("quota"),
            received: row.get("received"),
            last_renewal_time: row.get("last_renewal_time"),
        };

        Ok(address)
    }

    /// Update an address with new info
    pub async fn update_address(&mut self, address: &Address)
        -> Result<(), Box<dyn std::error::Error>> {
        // For now, just increment the received count
        let query = format!("
            UPDATE {}
            SET received = received + 1
            WHERE address = $1", &self.address_table
        );

        let _num_rows = sqlx::query(&query)
                             .bind(&address.address)
                             .execute(self.db)
                             .await?;

        Ok(())
    }

    /// Log a message to the logs table
    pub async fn log(&mut self, msg: String, recipient: &str, log_level: LogLevel)
        -> Result<(), Box<dyn std::error::Error>> {
        let is_debug = match log_level {
            LogLevel::Debug => true,
            _ => false,
        };

        let query = format!("
            INSERT INTO {} (user_id, address_id, msg, log_level, is_debug) VALUES
            ((SELECT user_id FROM {1} WHERE address = $1),
             (SELECT id FROM {1} WHERE address = $1), $2, $3, $4)",
            &self.log_table, &self.address_table
        );

        let _num_rows = sqlx::query(&query)
                             .bind(recipient)
                             .bind(&msg)
                             .bind(log_level as i32)
                             .bind(is_debug)
                             .execute(self.db)
                             .await?;

        Ok(())
    }
}
