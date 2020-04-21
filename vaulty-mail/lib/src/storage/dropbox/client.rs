use std::time::Duration;

use bytes::Bytes;
use futures::stream::Stream;
use reqwest::header::CONTENT_TYPE;

use super::api;

use crate::storage::client::{Client, ClientFuture};
use crate::storage::Error;

pub struct DropboxClient<'a> {
    token: &'a str,
    client: reqwest::Client,
}

impl<'a> DropboxClient<'a> {
    pub fn from_token(token: &'a str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(api::DROPBOX_REQUEST_TIMEOUT))
            .build()
            .unwrap();
        Self {
            token: token,
            client: client,
        }
    }

    #[inline]
    async fn request(
        &self,
        endpoint: api::Endpoint,
        body: reqwest::Body,
        args: Option<&str>,
        content_type: Option<&str>,
    ) -> Result<bytes::Bytes, Error> {
        let url = api::build_endpoint_url(endpoint);

        let mut req = self
            .client
            .post(reqwest::Url::parse(&url)?)
            .bearer_auth(&self.token)
            .header(CONTENT_TYPE, content_type.unwrap_or("application/json"))
            .body(body);

        if let Some(v) = args {
            req = req.header(api::DROPBOX_ARG_HEADER, v);
        }

        // Map response into an error if applicable
        let resp = api::map_status(req.send().await?);

        Ok(resp?.bytes().await?)
    }

    pub async fn list_folder(&self, path: &str) -> Result<api::ListFolderResult, Error> {
        let body = serde_json::json!({ "path": path }).to_string();
        let resp = self
            .request(api::Endpoint::ListFolder, body.into(), None, None)
            .await?;
        serde_json::from_slice(&resp).map_err(|e| e.into())
    }

    /// Create a folder in user's Dropbox
    /// This function does not return any API metadata
    pub async fn create_folder(&self, path: &str) -> Result<(), Error> {
        let body = serde_json::json!({ "path": path }).to_string();
        let _resp = self
            .request(api::Endpoint::CreateFolder, body.into(), None, None)
            .await?;
        Ok(())
    }

    /// Upload a file to a user's Dropbox
    /// This function does not return any API metadata
    pub async fn upload(&self, path: &str, data: Vec<u8>) -> Result<(), Error> {
        // Auto-rename the attachment if it exists
        let args = serde_json::json!({"path": path, "autorename": true}).to_string();
        let _resp = self
            .request(
                api::Endpoint::FileUpload,
                data.into(),
                Some(&args),
                Some("application/octet-stream"),
            )
            .await?;
        Ok(())
    }

    pub async fn search(&self, path: &str, query: &str) -> Result<api::SearchResult, Error> {
        let data = serde_json::json!({"path": path, "query": query}).to_string();
        let resp = self
            .request(api::Endpoint::Search, data.into(), None, None)
            .await?;
        serde_json::from_slice(&resp).map_err(|e| e.into())
    }
}

impl<'a> Client for DropboxClient<'a> {
    /// Upload a file to a user's Dropbox
    /// This function does not return any API metadata
    fn upload_stream(
        &self,
        path: &str,
        data: impl Stream<Item = Result<Bytes, crate::Error>> + Send + Sync + 'static,
    ) -> ClientFuture<'_, ()> {
        // Auto-rename the attachment if it exists
        let args = serde_json::json!({"path": path, "autorename": true}).to_string();
        let url = api::build_endpoint_url(api::Endpoint::FileUpload);

        Box::pin(async move {
            let mut req = self
                .client
                .post(reqwest::Url::parse(&url)?)
                .bearer_auth(&self.token)
                .header(CONTENT_TYPE, "application/octet-stream")
                .body(reqwest::Body::wrap_stream(data));

            req = req.header(api::DROPBOX_ARG_HEADER, args);

            // Map response into an error if applicable
            let _resp = api::map_status(req.send().await?)?.bytes().await?;

            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_list_folder() {
        let token = std::env::var("DROPBOX_TOKEN").expect("No Dropbox token found");
        let client = DropboxClient::from_token(&token);

        let result = client.list_folder("").await;

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_folder() {
        let token = std::env::var("DROPBOX_TOKEN").expect("No Dropbox token found");
        let client = DropboxClient::from_token(&token);

        let result = client.create_folder("/abcde").await;

        println!("{:?}", result);
    }

    #[tokio::test]
    async fn test_file_upload() {
        let token = std::env::var("DROPBOX_TOKEN").expect("No Dropbox token found");
        let client = DropboxClient::from_token(&token);
        let data = String::from("Hello there!").into_bytes();

        let result = client.upload("/vaulty_test.txt", data).await;

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    /// /vaulty/search1 -> "test/", "test123/"
    async fn test_search_folders() {
        let token = std::env::var("DROPBOX_TOKEN").expect("No Dropbox token found");
        let client = DropboxClient::from_token(&token);

        let result = client.search("/vaulty/search1", "test").await;

        println!("{:?}", result);
        assert!(result.is_ok());
    }

    #[tokio::test]
    /// /vaulty/search2 -> "test", "test123", "test/"
    async fn test_search_files_and_folders() {
        let token = std::env::var("DROPBOX_TOKEN").expect("No Dropbox token found");
        let client = DropboxClient::from_token(&token);

        let result = client.search("/vaulty/search2", "test").await;

        println!("{:?}", result);
        assert!(result.is_ok());
    }
}
