use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spreadsheet {
    pub spreadsheet_id: String,
    pub properties: SpreadsheetProperties,
    #[serde(default)]
    pub sheets: Vec<Sheet>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpreadsheetProperties {
    pub title: String,
    pub locale: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sheet {
    pub properties: SheetProperties,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetProperties {
    pub sheet_id: i64,
    pub title: String,
    pub index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueRange {
    pub range: String,
    pub major_dimension: Option<String>,
    #[serde(default)]
    pub values: Vec<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateValuesResponse {
    pub spreadsheet_id: String,
    pub updated_range: String,
    pub updated_rows: Option<i64>,
    pub updated_columns: Option<i64>,
    pub updated_cells: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppendValuesResponse {
    pub spreadsheet_id: String,
    pub table_range: Option<String>,
    pub updates: Option<UpdateValuesResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchGetValuesResponse {
    pub spreadsheet_id: String,
    #[serde(default)]
    pub value_ranges: Vec<ValueRange>,
}

/// Simplified sheet list for list-sheets command
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetListResponse {
    pub spreadsheet_id: String,
    pub title: String,
    pub sheets: Vec<SheetInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetInfo {
    pub sheet_id: i64,
    pub title: String,
    pub index: i64,
}

impl SheetListResponse {
    pub fn from_spreadsheet(spreadsheet: &Spreadsheet) -> Self {
        Self {
            spreadsheet_id: spreadsheet.spreadsheet_id.clone(),
            title: spreadsheet.properties.title.clone(),
            sheets: spreadsheet.sheets.iter().map(|s| SheetInfo {
                sheet_id: s.properties.sheet_id,
                title: s.properties.title.clone(),
                index: s.properties.index,
            }).collect(),
        }
    }
}

// Batch update types for sheet operations (different from Docs API)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetsBatchUpdateRequest {
    pub requests: Vec<SheetRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_sheet: Option<AddSheetRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update_sheet_properties: Option<UpdateSheetPropertiesRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete_sheet: Option<DeleteSheetRequest>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSheetRequest {
    pub properties: AddSheetProperties,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddSheetProperties {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSheetPropertiesRequest {
    pub properties: UpdateSheetProperties,
    /// Fields to update (e.g., "title" for rename)
    pub fields: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSheetProperties {
    /// Sheet ID to update
    pub sheet_id: i64,
    /// New title (for rename)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// New index (for reordering)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSheetRequest {
    pub sheet_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SheetsBatchUpdateResponse {
    pub spreadsheet_id: String,
    #[serde(default)]
    pub replies: Vec<serde_json::Value>,
}
