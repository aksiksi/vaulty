use std::collections::HashMap;

pub const MAX_EMAIL_SIZE: u64 = 5 * 1024 * 1024;
pub const MAX_ATTACHMENT_SIZE: u64 = 20 * 1024 * 1024;

// TODO: Migrate to file or DB lookup in `basic_auth`
pub const VAULTY_USER: &str = "admin";
pub const VAULTY_PASS: &str = "test123";

const DEFAULT_PORT: u16 = 7777;
const DEFAULT_DB_NAME: &str = "vaulty";
const DEFAULT_DB_USER: &str = "vaulty";

#[derive(Debug, Default)]
pub struct Config {
    pub port: u16,
    pub mailgun_key: Option<String>,
    pub db_name: String,
    pub db_host: String,
    pub db_user: String,
    pub db_password: Option<String>,
}

impl Config {
    pub fn load(path: Option<&str>) -> Self {
        let settings = vaulty::config::load_config(path);
        settings.into()
    }
}

impl From<HashMap<String, String>> for Config {
    fn from(settings: HashMap<String, String>) -> Self {
        let mut config = Self::default();

        config.port = settings.get("port")
                              .and_then(|p| p.parse::<u16>().ok())
                              .unwrap_or(DEFAULT_PORT);
        config.mailgun_key = settings.get("mailgun_key").map(String::from);
        config.db_host = settings.get("db_host")
                                 .unwrap_or(&"127.0.0.1".to_string())
                                 .to_string();
        config.db_name = settings.get("db_name")
                                 .unwrap_or(&DEFAULT_DB_NAME.to_string())
                                 .to_string();
        config.db_user = settings.get("db_user")
                                 .unwrap_or(&DEFAULT_DB_USER.to_string())
                                 .to_string();
        config.db_password = settings.get("db_password").map(String::from);

        config
    }
}
