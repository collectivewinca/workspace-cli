use crate::client::ApiClient;
use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Response from batchUpdate
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUpdateResponse {
    pub presentation_id: String,
    #[serde(default)]
    pub replies: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchUpdateRequest {
    requests: Vec<SlideRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SlideRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    create_slide: Option<CreateSlideRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    create_shape: Option<CreateShapeRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insert_text: Option<InsertTextRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateSlideRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    object_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    insertion_index: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    slide_layout_reference: Option<LayoutReference>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LayoutReference {
    predefined_layout: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateShapeRequest {
    object_id: String,
    shape_type: String,
    element_properties: ElementProperties,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ElementProperties {
    page_object_id: String,
    size: Size,
    transform: Transform,
}

#[derive(Debug, Clone, Serialize)]
pub struct Size {
    pub width: Dimension,
    pub height: Dimension,
}

#[derive(Debug, Clone, Serialize)]
pub struct Dimension {
    pub magnitude: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transform {
    pub scale_x: f64,
    pub scale_y: f64,
    pub translate_x: f64,
    pub translate_y: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct InsertTextRequest {
    object_id: String,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    insertion_index: Option<i32>,
}

/// Add a new slide to a presentation
pub async fn add_slide(
    client: &ApiClient,
    presentation_id: &str,
    index: Option<i32>,
    layout: Option<&str>,
) -> Result<BatchUpdateResponse> {
    let layout_reference = layout.map(|l| LayoutReference {
        predefined_layout: l.to_string(),
    });

    let request = BatchUpdateRequest {
        requests: vec![
            SlideRequest {
                create_slide: Some(CreateSlideRequest {
                    object_id: None,
                    insertion_index: index,
                    slide_layout_reference: layout_reference,
                }),
                create_shape: None,
                insert_text: None,
            },
        ],
    };

    let path = format!("/presentations/{}:batchUpdate", presentation_id);
    client.post(&path, &request).await
}

/// Add text to a slide at a specific position
pub async fn add_text(
    client: &ApiClient,
    presentation_id: &str,
    page_id: &str,
    text: &str,
    x_pt: f64,
    y_pt: f64,
    width_pt: f64,
    height_pt: f64,
) -> Result<BatchUpdateResponse> {
    // Generate a unique ID for the shape
    let shape_id = format!("textbox_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    let request = BatchUpdateRequest {
        requests: vec![
            // First create the shape (text box)
            SlideRequest {
                create_slide: None,
                create_shape: Some(CreateShapeRequest {
                    object_id: shape_id.clone(),
                    shape_type: "TEXT_BOX".to_string(),
                    element_properties: ElementProperties {
                        page_object_id: page_id.to_string(),
                        size: Size {
                            width: Dimension { magnitude: width_pt, unit: "PT".to_string() },
                            height: Dimension { magnitude: height_pt, unit: "PT".to_string() },
                        },
                        transform: Transform {
                            scale_x: 1.0,
                            scale_y: 1.0,
                            translate_x: x_pt,
                            translate_y: y_pt,
                            unit: "PT".to_string(),
                        },
                    },
                }),
                insert_text: None,
            },
            // Then insert text into the shape
            SlideRequest {
                create_slide: None,
                create_shape: None,
                insert_text: Some(InsertTextRequest {
                    object_id: shape_id,
                    text: text.to_string(),
                    insertion_index: Some(0),
                }),
            },
        ],
    };

    let path = format!("/presentations/{}:batchUpdate", presentation_id);
    client.post(&path, &request).await
}

/// Predefined slide layouts available in Google Slides
pub const SLIDE_LAYOUTS: &[&str] = &[
    "BLANK",
    "CAPTION_ONLY",
    "TITLE",
    "TITLE_AND_BODY",
    "TITLE_AND_TWO_COLUMNS",
    "TITLE_ONLY",
    "SECTION_HEADER",
    "SECTION_TITLE_AND_DESCRIPTION",
    "ONE_COLUMN_TEXT",
    "MAIN_POINT",
    "BIG_NUMBER",
];
