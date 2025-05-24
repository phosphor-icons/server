use crate::icons::{Category, FigmaCategory, IconStatus};
use serde::Deserialize;
use std::str::FromStr;
use thiserror::Error;

const APPSHEET_REGION: &str = "www.appsheet.com";
const APP_ID: &str = "14ed274a-6160-4aae-8ee2-9f746dc77f64";
const TABLE_NAME: &str = "Icon Inventory";

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct TableIcon {
    #[serde(default)]
    pub id: i32,
    #[serde(rename = "Row ID")]
    pub rid: String,
    pub name: String,
    #[serde(deserialize_with = "deserialize_string_or_none")]
    pub alias: Option<String>,
    #[serde(
        rename = "Codepoint",
        deserialize_with = "deserialize_string_to_opt_i32"
    )]
    pub code: Option<i32>,
    pub status: IconStatus,
    #[serde(
        rename = "Search Categories",
        deserialize_with = "deserialize_categories"
    )]
    pub search_categories: Vec<Category>,
    pub category: FigmaCategory,
    #[serde(deserialize_with = "deserialize_string_array")]
    pub tags: Vec<String>,
    #[serde(deserialize_with = "deserialize_string_or_none")]
    pub notes: Option<String>,
    #[serde(rename = "Release", deserialize_with = "deserialize_string_to_opt_f64")]
    pub released_at: Option<f64>,
    #[serde(
        rename = "Last Updated",
        deserialize_with = "deserialize_string_to_opt_f64"
    )]
    pub last_updated_at: Option<f64>,
    #[serde(
        rename = "Deprecated",
        deserialize_with = "deserialize_string_to_opt_f64"
    )]
    pub deprecated_at: Option<f64>,
    #[serde(deserialize_with = "deserialize_stringbool")]
    pub published: bool,
}

fn deserialize_stringbool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    match s.to_uppercase().as_str() {
        "Y" => Ok(true),
        "N" | "" => Ok(false),
        _ => {
            tracing::warn!("expected 'Y' or 'N', got '{s}'");
            Ok(false)
        }
    }
}

fn deserialize_string_or_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}

fn deserialize_string_array<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: String = String::deserialize(deserializer)?;
    let values: Vec<String> = value.split(", ").map(|s| s.to_string()).collect();
    Ok(values)
}

fn deserialize_string_to_opt_i32<'de, D>(deserializer: D) -> Result<Option<i32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    let i = s.parse::<i32>().map_err(serde::de::Error::custom)?;
    Ok(Some(i))
}

#[allow(unused)]
fn deserialize_string_to_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse::<f64>().map_err(serde::de::Error::custom)
}

fn deserialize_string_to_opt_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        return Ok(None);
    }
    let i = s.parse::<f64>().map_err(serde::de::Error::custom)?;
    Ok(Some(i))
}

fn deserialize_categories<'de, D>(deserializer: D) -> Result<Vec<Category>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let categories: String = String::deserialize(deserializer)?;
    let categories: Vec<&str> = categories.split(", ").collect();
    let mut result = Vec::new();
    for category in categories {
        match Category::from_str(&category) {
            Ok(cat) => result.push(cat),
            Err(_) => result.push(Category::Unknown),
        }
    }
    Ok(result)
}

pub struct TableClient;

impl std::fmt::Debug for TableClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableClient").finish()
    }
}

#[derive(Debug, Error)]
pub enum TableClientError {
    #[error("Missing GOOGLE_APPSHEET_APPLICATION_KEY")]
    MissingKey,
    #[error("Failed to perform Google API request")]
    BadRequest,
    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

impl TableClient {
    pub fn base_url() -> String {
        format!("https://{APPSHEET_REGION}/api/v2/apps/{APP_ID}/tables/{TABLE_NAME}/Action")
    }

    pub async fn sync() -> Result<Vec<TableIcon>, TableClientError> {
        let client = reqwest::Client::new();
        let url = TableClient::base_url();
        let access_key = std::env::var("GOOGLE_APPSHEET_APPLICATION_KEY")
            .map_err(|_| TableClientError::MissingKey)?;

        let response = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("ApplicationAccessKey", access_key)
            .json(&serde_json::json!({
                "Action": "Find",
                "Properties": {
                    "Locale": "en-US",
                }
            }))
            .send()
            .await
            .map_err(|_| TableClientError::BadRequest)?;

        response
            .json()
            .await
            .map_err(|_| TableClientError::ParseError("Failed to parse JSON".to_string()))
    }
}
