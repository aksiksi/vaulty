use std::error;
use std::io::{Read};
use std::fmt::Write;

use reqwest::blocking;
use reqwest::header::{CONTENT_TYPE};

mod api;

pub struct Client {
    token: String,
    client: blocking::Client,
}

impl Client {
    pub fn new() -> Self {
        Self {
            token: String::new(),
            client: blocking::Client::new(),
        }
    }

    pub fn from_token(token: &str) -> Self {
        let mut client = Self::new();
        client.set_token(token);
        client
    }

    pub fn set_token(&mut self, token: &str) {
        self.token.write_str(token).expect("Failed to save token");
    }

    #[inline]
    fn request(&self, endpoint: api::Endpoint,  body: &[u8],
               args: Option<&str>, content_type: Option<&str>) -> Result<String, Box<dyn error::Error>> {
        let url = api::build_endpoint_url(endpoint);

        let mut req = self.client
            .post(reqwest::Url::parse(&url)?)
            .bearer_auth(&self.token)
            .header(CONTENT_TYPE, content_type.unwrap_or("application/json"))
            .body(body.to_owned());

        if let Some(v) = args {
            req = req.header(api::DROPBOX_ARG_HEADER, v);
        }

        let resp = api::Error::map_status(req.send()?);

        let mut buf = String::new();
        resp?.read_to_string(&mut buf)?;

        Ok(buf)
    }

    pub fn list_folder(&self, path: &str) -> Result<api::ListFolderResult, Box<dyn error::Error>> {
        let body = serde_json::json!({"path": path}).to_string();
        let resp = self.request(api::Endpoint::ListFolder, body.as_bytes(), None, None)?;
        serde_json::from_str(&resp).map_err(|e| e.into())
    }

    pub fn create_folder(&self, path: &str) -> Result<api::CreateFolderResult, Box<dyn error::Error>> {
        let body = serde_json::json!({"path": path}).to_string();
        let resp = self.request(api::Endpoint::CreateFolder, body.as_bytes(), None, None)?;
        serde_json::from_str(&resp).map_err(|e| e.into())
    }

    pub fn upload(&self, path: &str, data: &[u8], rename: bool) -> Result<api::FileUploadResult, Box<dyn error::Error>> {
        let args = serde_json::json!({"path": path, "autorename": rename}).to_string();
        let resp = self.request(api::Endpoint::FileUpload, data, Some(&args), Some("application/octet-stream"))?;
        serde_json::from_str(&resp).map_err(|e| e.into())
    }

    pub fn search(&self, path: &str, query: &str) -> Result<api::SearchResult, Box<dyn error::Error>> {
        let data = serde_json::json!({"path": path, "query": query}).to_string();
        let resp = self.request(api::Endpoint::Search, data.as_bytes(), None, None)?;
        serde_json::from_str(&resp).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_client() -> Client {
        let token = std::env::var("DROPBOX_TOKEN").unwrap();
        Client::from_token(&token)
    }

    #[test]
    fn test_list_folder() {
        let client = get_client();
        let result = client.list_folder("");

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_folder() {
        let client = get_client();
        let result = client.create_folder("/abcde");

        println!("{:?}", result);
    }

    #[test]
    fn test_file_upload() {
        let client = get_client();
        let data = "Hello there!".as_bytes();
        let result = client.upload("/vaulty_test.txt", data, true);

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    /// /vaulty/search1 -> "test/", "test123/"
    fn test_search_folders() {
        let client = get_client();
        let result = client.search("/vaulty/search1", "test");

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[test]
    /// /vaulty/search2 -> "test", "test123", "test/"
    fn test_search_files_and_folders() {
        let client = get_client();
        let result = client.search("/vaulty/search2", "test");

        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
