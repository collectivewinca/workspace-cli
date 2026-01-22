use crate::client::ApiClient;
use crate::error::Result;
use crate::utils::base64::decode_base64url_string;
use crate::utils::html_to_md::smart_convert;
use super::types::{Message, MessagePart, MinimalMessage};

pub async fn get_message(client: &ApiClient, id: &str, format: &str) -> Result<Message> {
    let query = [("format", format)];
    client.get_with_query(&format!("/users/me/messages/{}", id), &query).await
}

/// Extract and decode the message body
pub fn extract_body(message: &Message) -> Option<String> {
    if let Some(ref payload) = message.payload {
        // Try to get body from payload directly
        if let Some(ref body) = payload.body {
            if let Some(ref data) = body.data {
                if !data.is_empty() {
                    if let Ok(decoded) = decode_base64url_string(data) {
                        return Some(decoded);
                    }
                }
            }
        }

        // Try to find text/plain or text/html in parts
        // Prefer text/plain over text/html for multipart/alternative
        if let Some(text) = find_text_part(&payload.parts, "text/plain") {
            return Some(text);
        }

        // Fallback to text/html if text/plain not found
        if let Some(html) = find_text_part(&payload.parts, "text/html") {
            return Some(html);
        }
    }
    None
}

fn find_text_part(parts: &[MessagePart], preferred_mime: &str) -> Option<String> {
    // Check if this is a multipart/alternative container
    for part in parts {
        let mime = part.mime_type.as_deref().unwrap_or("");

        // If we find multipart/alternative, search within it for the preferred type
        if mime == "multipart/alternative" {
            if let Some(text) = find_text_part(&part.parts, preferred_mime) {
                return Some(text);
            }
        }
    }

    // Look for the preferred MIME type in current level
    for part in parts {
        let mime = part.mime_type.as_deref().unwrap_or("");

        if mime == preferred_mime {
            if let Some(ref body) = part.body {
                if let Some(ref data) = body.data {
                    if !data.is_empty() {
                        if let Ok(decoded) = decode_base64url_string(data) {
                            return Some(decoded);
                        }
                    }
                }
            }
        }
    }

    // Recurse into nested parts that aren't multipart/alternative
    for part in parts {
        let mime = part.mime_type.as_deref().unwrap_or("");

        if mime.starts_with("multipart/") && mime != "multipart/alternative" {
            if let Some(text) = find_text_part(&part.parts, preferred_mime) {
                return Some(text);
            }
        }
    }

    None
}

/// Get header value by name
pub fn get_header(message: &Message, name: &str) -> Option<String> {
    message.payload.as_ref()?.headers.iter()
        .find(|h| h.name.eq_ignore_ascii_case(name))
        .map(|h| h.value.clone())
}

/// Get a message in minimal format (optimized for AI agents)
/// Returns only essential headers and plain text body
pub async fn get_message_minimal(client: &ApiClient, id: &str) -> Result<MinimalMessage> {
    let message = get_message(client, id, "full").await?;

    // Extract essential headers
    let from = get_header(&message, "From");
    let to = get_header(&message, "To");
    let subject = get_header(&message, "Subject");
    let date = get_header(&message, "Date");

    // Extract and convert body to plain text
    let body = extract_body(&message).map(|b| smart_convert(&b));

    Ok(MinimalMessage {
        id: message.id,
        thread_id: message.thread_id,
        from,
        to,
        subject,
        date,
        labels: message.label_ids,
        body,
    })
}

/// Information about an attachment
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentInfo {
    pub attachment_id: String,
    pub filename: String,
    pub mime_type: String,
    pub size: i64,
}

/// List attachments in a message
pub fn list_attachments(message: &Message) -> Vec<AttachmentInfo> {
    let mut attachments = Vec::new();
    if let Some(ref payload) = message.payload {
        collect_attachments(&payload.parts, &mut attachments);
    }
    attachments
}

fn collect_attachments(parts: &[MessagePart], attachments: &mut Vec<AttachmentInfo>) {
    for part in parts {
        // Check if this part is an attachment
        if let Some(ref body) = part.body {
            if let Some(ref attachment_id) = body.attachment_id {
                let filename = part.filename.clone().unwrap_or_else(|| "unnamed".to_string());
                let mime_type = part.mime_type.clone().unwrap_or_else(|| "application/octet-stream".to_string());
                let size = body.size.unwrap_or(0);

                attachments.push(AttachmentInfo {
                    attachment_id: attachment_id.clone(),
                    filename,
                    mime_type,
                    size,
                });
            }
        }

        // Recurse into nested parts
        collect_attachments(&part.parts, attachments);
    }
}

/// Downloaded attachment data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttachmentData {
    pub size: i64,
    pub data: String,  // Base64url encoded
}

/// Download an attachment by ID
pub async fn get_attachment(client: &ApiClient, message_id: &str, attachment_id: &str) -> Result<AttachmentData> {
    let path = format!("/users/me/messages/{}/attachments/{}", message_id, attachment_id);
    client.get(&path).await
}

/// Download and decode an attachment to bytes
pub async fn download_attachment(client: &ApiClient, message_id: &str, attachment_id: &str) -> Result<Vec<u8>> {
    let data = get_attachment(client, message_id, attachment_id).await?;

    // Try decoding as base64url (standard Gmail format)
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    URL_SAFE_NO_PAD.decode(&data.data)
        .map_err(|e| crate::error::WorkspaceError::Config(
            format!("Failed to decode attachment: {}", e)
        ))
}
