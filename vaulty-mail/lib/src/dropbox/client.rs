use std::error;

use reqwest::header::{CONTENT_TYPE};

use super::api;

pub struct Client {
    token: String,
    client: reqwest::Client,
}

impl Client {
    pub fn new() -> Self {
        Self {
            token: String::new(),
            client: reqwest::Client::new(),
        }
    }

    pub fn from_token(token: String) -> Self {
        Self {
            token: token,
            client: reqwest::Client::new(),
        }
    }

    #[inline]
    async fn request(&self, endpoint: api::Endpoint, body: reqwest::Body,
                     args: Option<&str>, content_type: Option<&str>) -> Result<bytes::Bytes, Box<dyn error::Error>> {
        let url = api::build_endpoint_url(endpoint);

        let mut req = self.client
            .post(reqwest::Url::parse(&url)?)
            .bearer_auth(&self.token)
            .header(CONTENT_TYPE, content_type.unwrap_or("application/json"))
            .body(body);

        if let Some(v) = args {
            req = req.header(api::DROPBOX_ARG_HEADER, v);
        }

        let resp = api::Error::map_status(req.send().await?);

        Ok(resp?.bytes().await?)
    }

    pub async fn list_folder(&self, path: &str) -> Result<api::ListFolderResult, Box<dyn error::Error>> {
        let body = serde_json::json!({"path": path}).to_string();
        let resp = self.request(api::Endpoint::ListFolder, body.into(), None, None).await?;
        serde_json::from_slice(&resp).map_err(|e| e.into())
    }

    pub async fn create_folder(&self, path: &str) -> Result<api::CreateFolderResult, Box<dyn error::Error>> {
        let body = serde_json::json!({"path": path}).to_string();
        let resp = self.request(api::Endpoint::CreateFolder, body.into(), None, None).await?;
        serde_json::from_slice(&resp).map_err(|e| e.into())
    }

    pub async fn upload(&self, path: &str, data: Vec<u8>, rename: bool) -> Result<api::FileUploadResult, Box<dyn error::Error>> {
        let args = serde_json::json!({"path": path, "autorename": rename}).to_string();
        let resp = self.request(api::Endpoint::FileUpload, data.into(), Some(&args), Some("application/octet-stream")).await?;
        serde_json::from_slice(&resp).map_err(|e| e.into())
    }

    pub async fn search(&self, path: &str, query: &str) -> Result<api::SearchResult, Box<dyn error::Error>> {
        let data = serde_json::json!({"path": path, "query": query}).to_string();
        let resp = self.request(api::Endpoint::Search, data.into(), None, None).await?;
        serde_json::from_slice(&resp).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_client() -> Client {
        let token = std::env::var("DROPBOX_TOKEN")
                             .expect("No Dropbox token found");
        Client::from_token(token)
    }

    #[tokio::test]
    async fn test_list_folder() {
        let client = get_client();
        let result = client.list_folder("").await;

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_folder() {
        let client = get_client();
        let result = client.create_folder("/abcde").await;

        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_file_upload() {
        let client = get_client();
        let data = String::from("Hello there!").into_bytes();
        let result = client.upload("/vaulty_test.txt", data, true).await;

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    /// /vaulty/search1 -> "test/", "test123/"
    async fn test_search_folders() {
        let client = get_client();
        let result = client.search("/vaulty/search1", "test").await;

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    /// /vaulty/search2 -> "test", "test123", "test/"
    async fn test_search_files_and_folders() {
        let client = get_client();
        let result = client.search("/vaulty/search2", "test").await;

        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
