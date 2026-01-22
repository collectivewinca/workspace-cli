use crate::client::ApiClient;
use crate::error::Result;
use super::types::Presentation;

/// Create a new presentation
pub async fn create_presentation(
    client: &ApiClient,
    title: &str,
) -> Result<Presentation> {
    let body = serde_json::json!({
        "title": title
    });

    client.post("/presentations", &body).await
}
