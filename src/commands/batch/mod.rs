use reqwest::Method;
use serde::{Deserialize, Serialize};
use crate::client::batch::{BatchClient, BatchRequest, BatchError};

/// Input format for a single batch request (agent-friendly JSON)
#[derive(Debug, Deserialize)]
pub struct BatchRequestInput {
    /// Unique identifier for this request (used to match responses)
    pub id: String,
    /// HTTP method (GET, POST, PUT, PATCH, DELETE)
    pub method: String,
    /// API path (e.g., "/users/me/messages/abc123")
    pub path: String,
    /// Optional request body for POST/PUT/PATCH
    #[serde(default)]
    pub body: Option<serde_json::Value>,
}

impl BatchRequestInput {
    /// Convert to internal BatchRequest format
    pub fn to_batch_request(&self) -> Result<BatchRequest, String> {
        let method = match self.method.to_uppercase().as_str() {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "PATCH" => Method::PATCH,
            "DELETE" => Method::DELETE,
            other => return Err(format!("Unsupported method: {}", other)),
        };

        Ok(BatchRequest {
            id: self.id.clone(),
            method,
            path: self.path.clone(),
            body: self.body.clone(),
        })
    }
}

/// Output format for batch results
#[derive(Debug, Serialize)]
pub struct BatchOutput {
    /// Overall status: "success" if all succeeded, "partial" if some failed
    pub status: &'static str,
    /// Successful results
    pub results: Vec<BatchResultOutput>,
    /// Failed requests
    pub errors: Vec<BatchErrorOutput>,
}

/// A successful result from the batch
#[derive(Debug, Serialize)]
pub struct BatchResultOutput {
    /// Request ID (matches input id)
    pub id: String,
    /// HTTP status code
    pub status: u16,
    /// Response body
    pub body: serde_json::Value,
}

/// An error from the batch
#[derive(Debug, Serialize)]
pub struct BatchErrorOutput {
    /// Request ID (matches input id)
    pub id: String,
    /// HTTP status code
    pub status: u16,
    /// Error message
    pub message: String,
}

/// Execute a batch of requests against a Google Workspace API
///
/// # Arguments
/// * `service` - The service name: "gmail", "drive", or "calendar"
/// * `inputs` - Vector of batch request inputs
/// * `access_token` - OAuth2 access token
///
/// # Returns
/// BatchOutput containing results and errors
pub async fn execute_batch(
    service: &str,
    inputs: Vec<BatchRequestInput>,
    access_token: &str,
) -> Result<BatchOutput, BatchError> {
    // Convert inputs to BatchRequests
    let mut requests = Vec::new();
    let mut parse_errors = Vec::new();

    for input in inputs {
        match input.to_batch_request() {
            Ok(req) => requests.push(req),
            Err(e) => parse_errors.push(BatchErrorOutput {
                id: input.id,
                status: 400,
                message: e,
            }),
        }
    }

    // Select batch client based on service
    let client = match service {
        "gmail" => BatchClient::gmail(),
        "drive" => BatchClient::drive(),
        "calendar" => BatchClient::calendar(),
        _ => return Err(BatchError::InvalidResponse(
            format!("Unknown service: {}. Use 'gmail', 'drive', or 'calendar'.", service)
        )),
    };

    // Execute batch (handles empty requests gracefully)
    let responses = client.execute(requests, access_token).await?;

    // Convert responses to output format
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for resp in responses {
        if resp.is_success() {
            results.push(BatchResultOutput {
                id: resp.id,
                status: resp.status,
                body: resp.body,
            });
        } else {
            // Extract error message from response body if available
            let message = resp.body
                .get("error")
                .and_then(|e| e.get("message"))
                .and_then(|m| m.as_str())
                .unwrap_or("Request failed")
                .to_string();
            errors.push(BatchErrorOutput {
                id: resp.id,
                status: resp.status,
                message,
            });
        }
    }

    // Add parse errors
    errors.extend(parse_errors);

    // Determine overall status
    let status = if errors.is_empty() {
        "success"
    } else if results.is_empty() {
        "error"
    } else {
        "partial"
    };

    Ok(BatchOutput {
        status,
        results,
        errors,
    })
}
