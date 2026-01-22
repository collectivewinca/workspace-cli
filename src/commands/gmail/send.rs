use crate::client::ApiClient;
use crate::error::Result;
use crate::utils::base64::encode_base64url_string;
use super::types::Message;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};

/// Represents a file attachment
pub struct Attachment {
    pub filename: String,
    pub content: Vec<u8>,
    pub mime_type: String,
}

pub struct ComposeParams {
    pub to: String,
    pub subject: String,
    pub body: String,
    pub from: Option<String>,
    pub cc: Option<String>,
    pub bcc: Option<String>,
    /// Message-ID of parent message (for replies)
    pub in_reply_to: Option<String>,
    /// Space-separated Message-IDs of thread history (for replies)
    pub references: Option<String>,
    /// Gmail thread ID (for replies)
    pub thread_id: Option<String>,
    /// Whether the body is HTML content
    pub is_html: bool,
    /// File attachments
    pub attachments: Vec<Attachment>,
}

pub async fn send_message(client: &ApiClient, params: ComposeParams) -> Result<Message> {
    let raw_email = build_raw_email(&params);
    let encoded = encode_base64url_string(&raw_email);

    // Include threadId for replies to maintain threading
    let request = if let Some(ref thread_id) = params.thread_id {
        serde_json::json!({
            "raw": encoded,
            "threadId": thread_id
        })
    } else {
        serde_json::json!({ "raw": encoded })
    };

    client.post("/users/me/messages/send", &request).await
}

pub async fn create_draft(client: &ApiClient, params: ComposeParams) -> Result<serde_json::Value> {
    let raw_email = build_raw_email(&params);
    let encoded = encode_base64url_string(&raw_email);

    // Include threadId for reply drafts to maintain threading
    let message = if let Some(ref thread_id) = params.thread_id {
        serde_json::json!({
            "raw": encoded,
            "threadId": thread_id
        })
    } else {
        serde_json::json!({ "raw": encoded })
    };

    let request = serde_json::json!({ "message": message });
    client.post("/users/me/drafts", &request).await
}

fn build_raw_email(params: &ComposeParams) -> String {
    let mut email = String::new();

    // Add Date header (RFC 2822 requirement)
    let now = chrono::Utc::now();
    email.push_str(&format!("Date: {}\r\n", now.to_rfc2822()));

    // Add Message-ID header (RFC 2822 recommendation)
    let message_id = format!("<{}.{}@workspace-cli>",
        now.timestamp(),
        uuid::Uuid::new_v4());
    email.push_str(&format!("Message-ID: {}\r\n", message_id));

    // Sanitize and add From header
    if let Some(ref from) = params.from {
        email.push_str(&format!("From: {}\r\n", sanitize_header(from)));
    }

    // Sanitize and add To header
    email.push_str(&format!("To: {}\r\n", sanitize_header(&params.to)));

    // Sanitize and add Cc header if present
    if let Some(ref cc) = params.cc {
        email.push_str(&format!("Cc: {}\r\n", sanitize_header(cc)));
    }

    // Sanitize and add Bcc header if present
    if let Some(ref bcc) = params.bcc {
        email.push_str(&format!("Bcc: {}\r\n", sanitize_header(bcc)));
    }

    // Add In-Reply-To header for replies (RFC 5322)
    if let Some(ref in_reply_to) = params.in_reply_to {
        email.push_str(&format!("In-Reply-To: {}\r\n", sanitize_header(in_reply_to)));
    }

    // Add References header for threading (RFC 5322)
    if let Some(ref references) = params.references {
        email.push_str(&format!("References: {}\r\n", sanitize_header(references)));
    }

    // Sanitize and add Subject header (RFC 2047 encode if non-ASCII)
    let subject = sanitize_header(&params.subject);
    if subject.is_ascii() {
        email.push_str(&format!("Subject: {}\r\n", subject));
    } else {
        // RFC 2047 encoded-word: =?charset?encoding?encoded_text?=
        let encoded = BASE64_STANDARD.encode(subject.as_bytes());
        email.push_str(&format!("Subject: =?UTF-8?B?{}?=\r\n", encoded));
    }

    email.push_str("MIME-Version: 1.0\r\n");

    // If we have attachments, use multipart/mixed format
    if !params.attachments.is_empty() {
        let boundary = format!("boundary_{}", uuid::Uuid::new_v4().to_string().replace('-', ""));
        email.push_str(&format!("Content-Type: multipart/mixed; boundary=\"{}\"\r\n", boundary));
        email.push_str("\r\n");

        // Body part
        email.push_str(&format!("--{}\r\n", boundary));
        if params.is_html {
            email.push_str("Content-Type: text/html; charset=utf-8\r\n");
        } else {
            email.push_str("Content-Type: text/plain; charset=utf-8\r\n");
        }
        email.push_str("\r\n");
        email.push_str(&params.body);
        email.push_str("\r\n");

        // Attachment parts
        for attachment in &params.attachments {
            email.push_str(&format!("--{}\r\n", boundary));
            email.push_str(&format!("Content-Type: {}; name=\"{}\"\r\n",
                attachment.mime_type,
                sanitize_header(&attachment.filename)));
            email.push_str("Content-Transfer-Encoding: base64\r\n");
            email.push_str(&format!("Content-Disposition: attachment; filename=\"{}\"\r\n",
                sanitize_header(&attachment.filename)));
            email.push_str("\r\n");

            // Base64 encode the attachment content with line breaks every 76 chars
            let encoded = BASE64_STANDARD.encode(&attachment.content);
            for chunk in encoded.as_bytes().chunks(76) {
                email.push_str(std::str::from_utf8(chunk).unwrap_or(""));
                email.push_str("\r\n");
            }
        }

        // Closing boundary
        email.push_str(&format!("--{}--\r\n", boundary));
    } else {
        // No attachments - simple email
        if params.is_html {
            email.push_str("Content-Type: text/html; charset=utf-8\r\n");
        } else {
            email.push_str("Content-Type: text/plain; charset=utf-8\r\n");
        }

        email.push_str("\r\n");
        email.push_str(&params.body);
    }

    email
}

/// Metadata extracted from an original message for constructing a reply
pub struct ReplyMetadata {
    /// Recipient for the reply (Reply-To or From of original)
    pub to: String,
    /// Cc recipients for reply-all (original To + Cc)
    pub cc: Option<String>,
    /// Subject with "Re: " prefix if not already present
    pub subject: String,
    /// Message-ID of the original message
    pub in_reply_to: String,
    /// Thread history (original References + Message-ID)
    pub references: String,
    /// Gmail thread ID
    pub thread_id: String,
}

/// Extract metadata from an original message needed to construct a reply
pub fn extract_reply_metadata(message: &Message) -> Option<ReplyMetadata> {
    let payload = message.payload.as_ref()?;
    let headers = &payload.headers;

    // Helper to get header value by name (case-insensitive)
    let get = |name: &str| -> Option<String> {
        headers.iter()
            .find(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.clone())
    };

    // Get reply-to address (prefer Reply-To, fall back to From)
    let to = get("Reply-To").or_else(|| get("From"))?;

    // Get subject, add "Re: " prefix if not already present
    let original_subject = get("Subject").unwrap_or_default();
    let subject = if original_subject.to_lowercase().starts_with("re:") {
        original_subject
    } else {
        format!("Re: {}", original_subject)
    };

    // Get Message-ID for In-Reply-To header
    let message_id = get("Message-ID")?;

    // Build References: original References (if any) + Message-ID
    let references = match get("References") {
        Some(refs) => format!("{} {}", refs, message_id),
        None => message_id.clone(),
    };

    // For reply-all: combine original To and Cc recipients
    let original_to = get("To");
    let original_cc = get("Cc");
    let cc = match (original_to, original_cc) {
        (Some(t), Some(c)) => Some(format!("{}, {}", t, c)),
        (Some(t), None) => Some(t),
        (None, Some(c)) => Some(c),
        (None, None) => None,
    };

    Some(ReplyMetadata {
        to,
        cc,
        subject,
        in_reply_to: message_id,
        references,
        thread_id: message.thread_id.clone(),
    })
}

/// Sanitize header values to prevent header injection attacks
/// Removes or replaces CR, LF, and CRLF sequences
fn sanitize_header(value: &str) -> String {
    value
        .replace('\r', "")
        .replace('\n', " ")
        .trim()
        .to_string()
}

/// Load an attachment from a file path
pub fn load_attachment(path: &str) -> std::io::Result<Attachment> {
    let path = std::path::Path::new(path);
    let filename = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("attachment")
        .to_string();

    let content = std::fs::read(path)?;

    // Detect MIME type based on file extension
    let mime_type = match path.extension().and_then(|e| e.to_str()) {
        Some("pdf") => "application/pdf",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        Some("txt") => "text/plain",
        Some("csv") => "text/csv",
        Some("html") | Some("htm") => "text/html",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("zip") => "application/zip",
        Some("tar") => "application/x-tar",
        Some("gz") | Some("gzip") => "application/gzip",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("webp") => "image/webp",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("mov") => "video/quicktime",
        Some("avi") => "video/x-msvideo",
        _ => "application/octet-stream",
    }.to_string();

    Ok(Attachment {
        filename,
        content,
        mime_type,
    })
}
