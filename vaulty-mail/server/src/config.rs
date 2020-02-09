pub const MAX_EMAIL_SIZE: u64 = 5 * 1024 * 1024;
pub const MAX_ATTACHMENT_SIZE: u64 = 20 * 1024 * 1024;

// TODO: Migrate to file or DB lookup in `basic_auth`
pub const VAULTY_USER: &str = "admin";
pub const VAULTY_PASS: &str = "test123";

#[derive(Debug)]
pub struct HttpArg {
    pub port: u16,
    pub mailgun_key: Option<String>,
}
