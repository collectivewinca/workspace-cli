use crate::client::ApiClient;
use crate::error::Result;
use super::types::{
    BatchUpdateRequest, BatchUpdateResponse, Request, InsertTextRequest, Location, Document,
    ReplaceAllTextRequest, SubstringMatchCriteria, InsertInlineImageRequest, InsertTableRequest,
    Size, Dimension,
};
use super::get::get_document;

/// Append text to the end of a document
pub async fn append_text(
    client: &ApiClient,
    document_id: &str,
    text: &str,
) -> Result<BatchUpdateResponse> {
    // First get the document to find the end index
    let doc = get_document(client, document_id).await?;
    let end_index = get_end_index(&doc);

    // Insert at end (index - 1 because we insert before the trailing newline)
    let insert_index = (end_index - 1).max(1);

    let request = BatchUpdateRequest {
        requests: vec![
            Request {
                insert_text: Some(InsertTextRequest {
                    text: format!("\n{}", text),
                    location: Location {
                        index: insert_index,
                        segment_id: None,
                    },
                }),
                replace_all_text: None,
                insert_inline_image: None,
                insert_table: None,
            },
        ],
    };

    let path = format!("/documents/{}:batchUpdate", document_id);
    client.post(&path, &request).await
}

/// Insert text at a specific index
pub async fn insert_text(
    client: &ApiClient,
    document_id: &str,
    text: &str,
    index: i64,
) -> Result<BatchUpdateResponse> {
    let request = BatchUpdateRequest {
        requests: vec![
            Request {
                insert_text: Some(InsertTextRequest {
                    text: text.to_string(),
                    location: Location {
                        index,
                        segment_id: None,
                    },
                }),
                replace_all_text: None,
                insert_inline_image: None,
                insert_table: None,
            },
        ],
    };

    let path = format!("/documents/{}:batchUpdate", document_id);
    client.post(&path, &request).await
}

fn get_end_index(doc: &Document) -> i64 {
    doc.body
        .as_ref()
        .and_then(|b| b.content.last())
        .and_then(|e| e.end_index)
        .unwrap_or(1)
}

/// Replace all occurrences of text in a document
pub async fn replace_text(
    client: &ApiClient,
    document_id: &str,
    find: &str,
    replace: &str,
    match_case: bool,
) -> Result<BatchUpdateResponse> {
    let request = BatchUpdateRequest {
        requests: vec![
            Request {
                insert_text: None,
                replace_all_text: Some(ReplaceAllTextRequest {
                    contains_text: SubstringMatchCriteria {
                        text: find.to_string(),
                        match_case,
                    },
                    replace_text: replace.to_string(),
                }),
                insert_inline_image: None,
                insert_table: None,
            },
        ],
    };

    let path = format!("/documents/{}:batchUpdate", document_id);
    client.post(&path, &request).await
}

/// Insert an inline image at a specific index
/// The image URI must be publicly accessible
pub async fn insert_image(
    client: &ApiClient,
    document_id: &str,
    uri: &str,
    index: i64,
    width_pt: Option<f64>,
    height_pt: Option<f64>,
) -> Result<BatchUpdateResponse> {
    let object_size = match (width_pt, height_pt) {
        (Some(w), Some(h)) => Some(Size {
            width: Some(Dimension { magnitude: w, unit: "PT".to_string() }),
            height: Some(Dimension { magnitude: h, unit: "PT".to_string() }),
        }),
        (Some(w), None) => Some(Size {
            width: Some(Dimension { magnitude: w, unit: "PT".to_string() }),
            height: None,
        }),
        (None, Some(h)) => Some(Size {
            width: None,
            height: Some(Dimension { magnitude: h, unit: "PT".to_string() }),
        }),
        (None, None) => None,
    };

    let request = BatchUpdateRequest {
        requests: vec![
            Request {
                insert_text: None,
                replace_all_text: None,
                insert_inline_image: Some(InsertInlineImageRequest {
                    location: Location {
                        index,
                        segment_id: None,
                    },
                    uri: uri.to_string(),
                    object_size,
                }),
                insert_table: None,
            },
        ],
    };

    let path = format!("/documents/{}:batchUpdate", document_id);
    client.post(&path, &request).await
}

/// Insert an inline image at the end of the document
pub async fn append_image(
    client: &ApiClient,
    document_id: &str,
    uri: &str,
    width_pt: Option<f64>,
    height_pt: Option<f64>,
) -> Result<BatchUpdateResponse> {
    // Get the document to find the end index
    let doc = get_document(client, document_id).await?;
    let end_index = get_end_index(&doc);
    let insert_index = (end_index - 1).max(1);

    insert_image(client, document_id, uri, insert_index, width_pt, height_pt).await
}

/// Insert a table at a specific index
pub async fn insert_table(
    client: &ApiClient,
    document_id: &str,
    rows: i64,
    columns: i64,
    index: i64,
) -> Result<BatchUpdateResponse> {
    let request = BatchUpdateRequest {
        requests: vec![
            Request {
                insert_text: None,
                replace_all_text: None,
                insert_inline_image: None,
                insert_table: Some(InsertTableRequest {
                    rows,
                    columns,
                    location: Location {
                        index,
                        segment_id: None,
                    },
                }),
            },
        ],
    };

    let path = format!("/documents/{}:batchUpdate", document_id);
    client.post(&path, &request).await
}

/// Insert a table at the end of the document
pub async fn append_table(
    client: &ApiClient,
    document_id: &str,
    rows: i64,
    columns: i64,
) -> Result<BatchUpdateResponse> {
    // Get the document to find the end index
    let doc = get_document(client, document_id).await?;
    let end_index = get_end_index(&doc);
    let insert_index = (end_index - 1).max(1);

    insert_table(client, document_id, rows, columns, insert_index).await
}
