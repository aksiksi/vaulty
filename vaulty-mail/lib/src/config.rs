use std::collections::HashMap;

pub const DEFAULT_CONFIG_PATH: &str = "/etc/vaulty/vaulty.toml";
const ENV_PREFIX: &str = "VAULTY_";

pub const MAX_EMAIL_SIZE: u64 = 5 * 1024 * 1024;
pub const MAX_ATTACHMENT_SIZE: u64 = 20 * 1024 * 1024;

pub const DEFAULT_VAULTY_USER: &str = "admin";
pub const DEFAULT_VAULTY_PASS: &str = "test123";

const DEFAULT_PORT: u16 = 7777;
const DEFAULT_DB_NAME: &str = "vaulty";
const DEFAULT_DB_USER: &str = "vaulty";

#[derive(Clone, Debug, Default)]
pub struct Config {
    pub port: u16,
    pub mailgun_key: Option<String>,
    pub max_email_size: u64,
    pub max_attachment_size: u64,
    pub vaulty_user: String,
    pub vaulty_pass: String,
    pub db_name: String,
    pub db_host: String,
    pub db_user: String,
    pub db_password: Option<String>,
}

impl Config {
    /// Loads Vaulty config from filesystem and merges it with any
    /// environment variables prefixed with VAULTY_.
    ///
    /// This function will panic on error.
    ///
    /// See sample config file in `examples` for valid keys.
    pub fn load(path: Option<&str>) -> Self {
        let mut settings = config::Config::default();

        settings
            .merge(config::File::with_name(path.unwrap_or(DEFAULT_CONFIG_PATH)))
            .unwrap()
            .merge(config::Environment::with_prefix(ENV_PREFIX))
            .unwrap();

        Self::from(settings.try_into::<HashMap<String, String>>().unwrap())
    }
}

impl From<HashMap<String, String>> for Config {
    fn from(settings: HashMap<String, String>) -> Self {
        let mut config = Self::default();

        config.port = settings
            .get("port")
            .and_then(|p| p.parse::<u16>().ok())
            .unwrap_or(DEFAULT_PORT);
        config.mailgun_key = settings.get("mailgun_key").map(String::from);
        config.max_email_size = settings
            .get("max_email_size")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(MAX_EMAIL_SIZE);
        config.max_attachment_size = settings
            .get("max_attachment_size")
            .and_then(|p| p.parse::<u64>().ok())
            .unwrap_or(MAX_ATTACHMENT_SIZE);
        config.vaulty_user = settings
            .get("vaulty_user")
            .unwrap_or(&DEFAULT_VAULTY_USER.to_string())
            .to_string();
        config.vaulty_pass = settings
            .get("vaulty_pass")
            .unwrap_or(&DEFAULT_VAULTY_PASS.to_string())
            .to_string();
        config.db_host = settings
            .get("db_host")
            .unwrap_or(&"127.0.0.1".to_string())
            .to_string();
        config.db_name = settings
            .get("db_name")
            .unwrap_or(&DEFAULT_DB_NAME.to_string())
            .to_string();
        config.db_user = settings
            .get("db_user")
            .unwrap_or(&DEFAULT_DB_USER.to_string())
            .to_string();
        config.db_password = settings.get("db_password").map(String::from);

        config
    }
}
