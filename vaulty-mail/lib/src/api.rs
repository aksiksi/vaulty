/// Contains API-related struct definitions that are shared between server
/// and client.
use serde::{Deserialize, Serialize};

/// JSON API response from Vaulty server.
///
/// Indicates if the operation succeeded and includes information about
/// the operation.
///
/// If email has attachments, the last attachment will return the full info.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ServerResult {
    pub success: bool,
    pub message: Option<String>,
    pub storage_backend: Option<crate::storage::Backend>,
    pub num_attachments: Option<i32>,
    pub error: Option<crate::Error>,
}
