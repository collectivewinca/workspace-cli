use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Gmail filter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub id: Option<String>,
    pub criteria: FilterCriteria,
    pub action: FilterAction,
}

/// Filter matching criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterCriteria {
    /// Match emails from this sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Match emails to this recipient
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Match emails with this subject
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    /// Match emails containing this query (Gmail search syntax)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    /// Match emails NOT containing this query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negated_query: Option<String>,
    /// Match emails with attachments
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_attachment: Option<bool>,
    /// Exclude chats
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_chats: Option<bool>,
    /// Match emails larger than this size (bytes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<i64>,
    /// Size comparison operator: "larger" or "smaller"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_comparison: Option<String>,
}

impl Default for FilterCriteria {
    fn default() -> Self {
        Self {
            from: None,
            to: None,
            subject: None,
            query: None,
            negated_query: None,
            has_attachment: None,
            exclude_chats: None,
            size: None,
            size_comparison: None,
        }
    }
}

/// Actions to perform on matching emails
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterAction {
    /// Add these label IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub add_label_ids: Vec<String>,
    /// Remove these label IDs
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub remove_label_ids: Vec<String>,
    /// Forward to this email address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forward: Option<String>,
}

impl Default for FilterAction {
    fn default() -> Self {
        Self {
            add_label_ids: Vec::new(),
            remove_label_ids: Vec::new(),
            forward: None,
        }
    }
}

/// Response from listing filters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListFiltersResponse {
    #[serde(default)]
    pub filter: Vec<Filter>,
}

/// List all filters for the user
pub async fn list_filters(client: &ApiClient) -> Result<ListFiltersResponse> {
    client.get("/users/me/settings/filters").await
}

/// Get a specific filter by ID
pub async fn get_filter(client: &ApiClient, filter_id: &str) -> Result<Filter> {
    let path = format!("/users/me/settings/filters/{}", urlencoding::encode(filter_id));
    client.get(&path).await
}

/// Create a new filter
pub async fn create_filter(client: &ApiClient, filter: &Filter) -> Result<Filter> {
    client.post("/users/me/settings/filters", filter).await
}

/// Delete a filter by ID
pub async fn delete_filter(client: &ApiClient, filter_id: &str) -> Result<()> {
    let path = format!("/users/me/settings/filters/{}", urlencoding::encode(filter_id));
    client.delete(&path).await
}

/// Build a filter from common parameters
pub fn build_filter(
    from: Option<&str>,
    to: Option<&str>,
    subject: Option<&str>,
    query: Option<&str>,
    has_attachment: Option<bool>,
    add_labels: Vec<String>,
    remove_labels: Vec<String>,
    forward_to: Option<&str>,
    skip_inbox: bool,
    mark_read: bool,
    star: bool,
    archive: bool,
    trash: bool,
) -> Filter {
    let criteria = FilterCriteria {
        from: from.map(String::from),
        to: to.map(String::from),
        subject: subject.map(String::from),
        query: query.map(String::from),
        has_attachment,
        ..Default::default()
    };

    let mut add_label_ids = add_labels;
    let mut remove_label_ids = remove_labels;

    // Handle convenience flags
    if skip_inbox || archive {
        remove_label_ids.push("INBOX".to_string());
    }
    if mark_read {
        remove_label_ids.push("UNREAD".to_string());
    }
    if star {
        add_label_ids.push("STARRED".to_string());
    }
    if trash {
        add_label_ids.push("TRASH".to_string());
    }

    let action = FilterAction {
        add_label_ids,
        remove_label_ids,
        forward: forward_to.map(String::from),
    };

    Filter {
        id: None,
        criteria,
        action,
    }
}
