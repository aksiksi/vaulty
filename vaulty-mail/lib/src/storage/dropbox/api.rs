use crate::storage::Error;

use reqwest::StatusCode;

use serde::Deserialize;

pub const DROPBOX_ARG_HEADER: &str = "Dropbox-API-Arg";
pub const DROPBOX_BASE_API: &str = "https://api.dropboxapi.com/2/";
pub const DROPBOX_BASE_CONTENT: &str = "https://content.dropboxapi.com/2/";

// Request timeout, in seconds
pub(crate) const DROPBOX_REQUEST_TIMEOUT: u64 = 30;

/// Map possible Dropbox API errors to generic storage backend error
pub fn map_status(resp: reqwest::Response) -> Result<reqwest::Response, Error> {
    let err = resp.error_for_status_ref();

    if let Err(e) = err {
        let status = e.status().unwrap();
        let msg = e.to_string();

        match status {
            StatusCode::BAD_REQUEST => Err(Error::BadInput(msg)),
            StatusCode::FORBIDDEN => Err(Error::TokenExpired(msg)),
            StatusCode::CONFLICT => Err(Error::BadEndpoint(msg)),
            StatusCode::TOO_MANY_REQUESTS => Err(Error::RateLimited(msg)),
            _ => Err(Error::Internal(msg)),
        }
    } else {
        Ok(resp)
    }
}

pub enum Endpoint {
    ListFolder,
    CreateFolder,
    FileUpload,
    Search,
}

#[derive(Deserialize, Debug)]
#[serde(tag = ".tag")]
pub enum SearchResultEntry {
    #[serde(rename = "folder")]
    Folder {
        name: String,
        path_lower: String,
        path_display: String,
        id: String,
    },
    #[serde(rename = "file")]
    File {
        name: String,
        id: String,
        size: usize,
        server_modified: String,
        path_lower: String,
        path_display: String,
        content_hash: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct SearchResultSingle {
    metadata: SearchResultEntry,
}

#[derive(Deserialize, Debug)]
pub struct SearchResult {
    pub matches: Vec<SearchResultSingle>,
    pub more: bool,
}

#[derive(Deserialize, Debug)]
pub struct ListFolderResult {
    pub entries: Vec<SearchResultEntry>,
    pub has_more: bool,
}

#[derive(Deserialize, Debug)]
pub struct CreateFolderResult {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct FileUploadResult {
    name: String,
    id: String,
    size: usize,
    server_modified: String,
    path_lower: String,
    path_display: String,
    content_hash: String,
}

#[inline]
pub fn build_endpoint_url(endpoint: Endpoint) -> String {
    match endpoint {
        Endpoint::ListFolder => format!("{}{}", DROPBOX_BASE_API, "files/list_folder"),
        Endpoint::CreateFolder => format!("{}{}", DROPBOX_BASE_API, "files/create_folder_v2"),
        Endpoint::FileUpload => format!("{}{}", DROPBOX_BASE_CONTENT, "files/upload"),
        Endpoint::Search => format!("{}{}", DROPBOX_BASE_API, "files/search"),
    }
}
