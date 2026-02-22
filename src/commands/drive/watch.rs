use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Watch channel for receiving push notifications
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Channel {
    /// A UUID identifying this channel
    pub id: String,
    /// An opaque ID identifying the resource being watched
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// The type of delivery mechanism (must be "web_hook")
    pub r#type: String,
    /// The URL to send notifications to
    pub address: String,
    /// Token sent with notifications to verify the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// Unix timestamp (ms) when channel expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration: Option<i64>,
    /// The kind of resource being watched
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Parameters for watching a file
pub struct WatchFileParams {
    /// File ID to watch
    pub file_id: String,
    /// Webhook URL to receive notifications (must be HTTPS)
    pub webhook_url: String,
    /// Optional token to include in notifications for verification
    pub token: Option<String>,
    /// Optional expiration time in milliseconds from epoch
    pub expiration: Option<i64>,
}

/// Parameters for watching changes to the user's Drive
pub struct WatchChangesParams {
    /// Start page token (get from changes.getStartPageToken)
    pub page_token: String,
    /// Webhook URL to receive notifications (must be HTTPS)
    pub webhook_url: String,
    /// Optional token to include in notifications for verification
    pub token: Option<String>,
    /// Optional expiration time in milliseconds from epoch
    pub expiration: Option<i64>,
}

/// Response containing the start page token
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPageTokenResponse {
    pub start_page_token: String,
    pub kind: Option<String>,
}

/// Get the starting page token for watching changes
pub async fn get_start_page_token(client: &ApiClient) -> Result<StartPageTokenResponse> {
    client.get("/changes/startPageToken").await
}

/// Watch for changes to the user's Drive
pub async fn watch_changes(client: &ApiClient, params: WatchChangesParams) -> Result<Channel> {
    let channel_id = uuid::Uuid::new_v4().to_string();

    let mut body = serde_json::json!({
        "id": channel_id,
        "type": "web_hook",
        "address": params.webhook_url
    });

    if let Some(token) = params.token {
        body["token"] = serde_json::json!(token);
    }

    if let Some(exp) = params.expiration {
        body["expiration"] = serde_json::json!(exp);
    }

    let path = format!("/changes/watch?pageToken={}", urlencoding::encode(&params.page_token));
    client.post(&path, &body).await
}

/// Watch for changes to a specific file
pub async fn watch_file(client: &ApiClient, params: WatchFileParams) -> Result<Channel> {
    let channel_id = uuid::Uuid::new_v4().to_string();

    let mut body = serde_json::json!({
        "id": channel_id,
        "type": "web_hook",
        "address": params.webhook_url
    });

    if let Some(token) = params.token {
        body["token"] = serde_json::json!(token);
    }

    if let Some(exp) = params.expiration {
        body["expiration"] = serde_json::json!(exp);
    }

    let path = format!("/files/{}/watch", urlencoding::encode(&params.file_id));
    client.post(&path, &body).await
}

/// Stop a watch channel (stop receiving notifications)
pub async fn stop_channel(client: &ApiClient, channel_id: &str, resource_id: &str) -> Result<()> {
    let body = serde_json::json!({
        "id": channel_id,
        "resourceId": resource_id
    });

    client.post("/channels/stop", &body).await
}

/// List recent changes to the user's Drive
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangesResponse {
    pub kind: Option<String>,
    #[serde(default)]
    pub changes: Vec<Change>,
    pub new_start_page_token: Option<String>,
    pub next_page_token: Option<String>,
}

/// A change to a file in Drive
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Change {
    pub kind: Option<String>,
    pub change_type: Option<String>,
    pub time: Option<String>,
    pub removed: Option<bool>,
    pub file_id: Option<String>,
    pub file: Option<super::types::File>,
}

/// List changes since a page token
pub async fn list_changes(client: &ApiClient, page_token: &str) -> Result<ChangesResponse> {
    let path = format!("/changes?pageToken={}", urlencoding::encode(page_token));
    client.get(&path).await
}
