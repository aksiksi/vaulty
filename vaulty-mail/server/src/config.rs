#[derive(Debug)]
pub struct FilterArg<'a> {
    pub recipient: &'a str,
    pub sender: &'a str,
}

#[derive(Debug)]
pub struct HttpArg<'a> {
    pub port: u16,
    pub mailgun_key: Option<&'a str>,
}
