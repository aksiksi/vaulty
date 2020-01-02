use std::error;
use std::io::Read;

use reqwest::blocking;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};

use serde::Deserialize;

struct Client {
    auth: String,
    client: blocking::Client,
    base_url: &'static str,
}

#[derive(Deserialize, Debug)]
struct ListFolder {
    entries: Vec<ListFolderEntry>,
    has_more: bool,
}

#[derive(Deserialize, Debug)]
struct ListFolderEntry {
    #[serde(rename = ".tag")]
    tag: String,
    name: String,
    path_lower: String,
    path_display: String,
    id: String,
}

impl Client {
    fn new(token: String) -> Self {
        Self {
            auth: format!("Bearer {}", token),
            client: blocking::Client::new(),
            base_url: "https://api.dropboxapi.com/2/",
        }
    }

    #[inline]
    fn request(&self, endpoint: &str, args: String) -> Result<String, Box<dyn error::Error>> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut resp = self.client
            .post(reqwest::Url::parse(&url)?)
            .header(AUTHORIZATION, &self.auth)
            .header(CONTENT_TYPE, "application/json")
            .body(args)
            .send()?
            .error_for_status()?;

        let mut buf = String::new();
        resp.read_to_string(&mut buf)?;

        Ok(buf)
    }

    pub fn list_folder(&self, path: &str) -> Result<ListFolder, Box<dyn error::Error>> {
        let args = serde_json::json!({"path": path}).to_string();
        let resp = self.request("files/list_folder", args)?;

        // TODO: Can we handle error conversion more robustly?
        match serde_json::from_str(&resp) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string().into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_client() -> Client {
        let token = std::env::var("DROPBOX_TOKEN").unwrap();
        Client::new(token)
    }

    #[test]
    fn test_list_folder() {
        let client = get_client();
        let result = client.list_folder("");
        assert!(result.is_ok());
        println!("{:?}", result.unwrap());
    }
}
