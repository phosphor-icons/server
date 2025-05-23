use crate::icons::Icon;
use thiserror::Error;

const APPSHEET_REGION: &str = "www.appsheet.com";
const APP_ID: &str = "14ed274a-6160-4aae-8ee2-9f746dc77f64";
const TABLE_NAME: &str = "Icon Inventory";

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

    pub async fn sync() -> Result<Vec<Icon>, TableClientError> {
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
