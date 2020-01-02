use std::error;
use std::fmt;
use std::io::Read;

use reqwest::blocking;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;

use serde::{Serialize, Deserialize};

const DROPBOX_ARG_HEADER: &str = "Dropbox-API-Arg";
const DROPBOX_BASE_API: &str = "https://api.dropboxapi.com/2/";
const DROPBOX_BASE_CONTENT: &str = "https://content.dropboxapi.com/2/";

#[derive(Debug)]
enum Error {
    BadInput,
    TokenExpired,
    Endpoint,
    RateLimited,
    Internal(u16),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::BadInput => f.write_str("BadInput"),
            Error::TokenExpired => f.write_str("TokenExpired"),
            Error::Endpoint => f.write_str("Endpoint"),
            Error::RateLimited => f.write_str("RateLimited"),
            Error::Internal(_) => f.write_str("Internal Error"),
        }
    }
}

impl error::Error for Error {}

enum Endpoint {
    ListFolder,
    FileUpload,
    CreateFolder,
}

struct Client {
    token: String,
    client: blocking::Client,
}

#[derive(Deserialize, Debug)]
struct ListFolderResult {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct FileUploadResult {
    name: String,
    id: String,
    size: usize,
    server_modified: String,
    path_lower: String,
    path_display: String,
    content_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateFolderResult {
    name: String,
}

impl Client {
    fn new(token: String) -> Self {
        Self {
            token: token,
            client: blocking::Client::new(),
        }
    }

    #[inline]
    fn build_endpoint_url(endpoint: Endpoint) -> String {
        match endpoint {
            Endpoint::ListFolder => format!("{}{}", DROPBOX_BASE_API, "files/list_folder"),
            Endpoint::CreateFolder => format!("{}{}", DROPBOX_BASE_API, "files/create_folder_v2"),
            Endpoint::FileUpload => format!("{}{}", DROPBOX_BASE_CONTENT, "files/upload"),
        }
    }

    #[inline]
    fn request(&self, endpoint: Endpoint,  body: &[u8],
               args: Option<&str>, content_type: Option<&str>) -> Result<String, Box<dyn error::Error>> {
        let url = Self::build_endpoint_url(endpoint);

        let mut req = self.client
            .post(reqwest::Url::parse(&url)?)
            .bearer_auth(&self.token)
            .header(CONTENT_TYPE, content_type.unwrap_or("application/json"))
            .body(body.to_owned());

        if let Some(v) = args {
            req = req.header(DROPBOX_ARG_HEADER, v);
        }

        let mut resp = req.send()?;
        let status = resp.status();

        // Map possible Dropbox API errors
        let resp = match status {
            StatusCode::OK => Ok(resp),
            StatusCode::BAD_REQUEST => Err(Error::BadInput),
            StatusCode::FORBIDDEN => Err(Error::TokenExpired),
            StatusCode::CONFLICT => Err(Error::Endpoint),
            StatusCode::TOO_MANY_REQUESTS => Err(Error::RateLimited),
            _ => Err(Error::Internal(status.as_u16())),
        };

        let mut buf = String::new();
        resp?.read_to_string(&mut buf)?;

        Ok(buf)
    }

    pub fn list_folder(&self, path: &str) -> Result<ListFolderResult, Box<dyn error::Error>> {
        let body = serde_json::json!({"path": path}).to_string();
        let resp = self.request(Endpoint::ListFolder, body.as_bytes(), None, None)?;
        serde_json::from_str(&resp).map_err(|e| e.into())
    }

    pub fn create_folder(&self, path: &str) -> Result<CreateFolderResult, Box<dyn error::Error>> {
        let body = serde_json::json!({"path": path}).to_string();
        let resp = self.request(Endpoint::CreateFolder, body.as_bytes(), None, None)?;
        serde_json::from_str(&resp).map_err(|e| e.into())
    }

    pub fn upload(&self, path: &str, data: &[u8], rename: bool) -> Result<FileUploadResult, Box<dyn error::Error>> {
        let args = serde_json::json!({"path": path, "autorename": rename}).to_string();
        let resp = self.request(Endpoint::FileUpload, data, Some(&args), Some("application/octet-stream"))?;
        serde_json::from_str(&resp).map_err(|e| e.into())
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
}
