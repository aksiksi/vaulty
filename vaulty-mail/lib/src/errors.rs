#[derive(Debug)]
pub struct VaultyError {
    pub msg: String,
}

impl std::fmt::Display for VaultyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl std::error::Error for VaultyError {}
