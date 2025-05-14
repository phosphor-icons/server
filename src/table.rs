use crate::icons::{Category, FigmaCategory, Icon, IconStatus};
use gcloud_sdk::{
    self,
    google::area120::tables::v1alpha1::{
        tables_service_client::TablesServiceClient, ListRowsRequest, Row,
    },
    prost_types, tonic, GoogleApi, GoogleAuthMiddleware,
};
use thiserror::Error;

const TABLE_ID: &str = "asHu4ng7CDp49R5xQmb4sB";
const TABLES_API_URL: &str = "https://area120tables.googleapis.com";
const TABLES_SCOPE: &str = "https://www.googleapis.com/auth/tables";
const CLOUD_SCOPE: &str = "https://www.googleapis.com/auth/cloud-platform";

pub struct TableClient {
    client: GoogleApi<TablesServiceClient<GoogleAuthMiddleware>>,
}

impl std::fmt::Debug for TableClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TableClient").finish()
    }
}

pub enum TableColumn {
    Rid,
    Name,
    Status,
    Category,
    SearchCategories,
    Tags,
    Notes,
    ReleasedAt,
    LastUpdatedAt,
    DeprecatedAt,
    Published,
    Alias,
    Code,
}

impl TableColumn {
    fn as_str(&self) -> &str {
        match self {
            TableColumn::Rid => "RID",
            TableColumn::Name => "Name",
            TableColumn::Status => "Status",
            TableColumn::Category => "Category",
            TableColumn::SearchCategories => "Search Categories",
            TableColumn::Tags => "Tags",
            TableColumn::Notes => "Notes",
            TableColumn::ReleasedAt => "Release",
            TableColumn::LastUpdatedAt => "Last Updated",
            TableColumn::DeprecatedAt => "Deprecated",
            TableColumn::Published => "Published",
            TableColumn::Alias => "Alias",
            TableColumn::Code => "Codepoint",
        }
    }
}

#[derive(Debug, Error)]
pub enum TableClientError {
    #[error("Failed to create Google API client: {0}")]
    ClientInitialization(#[from] gcloud_sdk::error::Error),
    #[error("Failed to perform Google API request: {0}")]
    BadRequest(#[from] tonic::Status),
    #[error("Failed to parse response: {0}")]
    ParseError(String),
}

impl TableClient {
    pub async fn init() -> Result<Self, TableClientError> {
        let client = GoogleApi::from_function_with_scopes(
            TablesServiceClient::new,
            TABLES_API_URL,
            None,
            vec![CLOUD_SCOPE.into(), TABLES_SCOPE.into()],
        )
        .await?;

        Ok(TableClient { client })
    }

    pub async fn sync(&self) -> Result<Vec<Icon>, TableClientError> {
        let mut icons: Vec<Icon> = Vec::new();
        let mut page_token = "".to_string();

        loop {
            let rows = self
                .client
                .get()
                .list_rows(tonic::Request::new(ListRowsRequest {
                    parent: format!("tables/{TABLE_ID}"),
                    page_size: 1000,
                    page_token,
                    ..Default::default()
                }))
                .await?;

            let inner = rows.into_inner();
            page_token = inner.next_page_token;

            for row in inner.rows.into_iter() {
                let icon = Icon::try_from(row).unwrap();
                icons.push(icon);
            }

            if page_token.is_empty() {
                break;
            }
        }

        Ok(icons)
    }
}

impl TryFrom<Row> for Icon {
    type Error = TableClientError;
    fn try_from(row: Row) -> Result<Self, Self::Error> {
        let rid = row
            .values
            .get(TableColumn::Rid.as_str())
            .and_then(as_string)
            .ok_or_else(|| TableClientError::ParseError("Missing RID".to_string()))?;
        let name = row
            .values
            .get(TableColumn::Name.as_str())
            .and_then(as_string)
            .ok_or_else(|| TableClientError::ParseError("Missing Name".to_string()))?;
        let alias = row
            .values
            .get(TableColumn::Alias.as_str())
            .and_then(as_string);
        let code = row.values.get(TableColumn::Code.as_str()).and_then(as_int);
        let status = row
            .values
            .get(TableColumn::Status.as_str())
            .and_then(as_status)
            .unwrap_or(IconStatus::None);
        let category = row
            .values
            .get(TableColumn::SearchCategories.as_str())
            .map(as_search_categories)
            .unwrap_or_default();
        let figma_category = row
            .values
            .get(TableColumn::Category.as_str())
            .and_then(as_category)
            .unwrap_or(FigmaCategory::Unknown);
        let tags = row
            .values
            .get(TableColumn::Tags.as_str())
            .map(as_string_array)
            .unwrap_or_default();
        let notes = row
            .values
            .get(TableColumn::Notes.as_str())
            .and_then(as_string);
        let released_at = row
            .values
            .get(TableColumn::ReleasedAt.as_str())
            .and_then(as_float);
        let last_updated_at = row
            .values
            .get(TableColumn::LastUpdatedAt.as_str())
            .and_then(as_float);
        let published = row
            .values
            .get(TableColumn::Published.as_str())
            .map(as_bool)
            .unwrap_or_default();
        let deprecated_at = row
            .values
            .get(TableColumn::DeprecatedAt.as_str())
            .and_then(as_float);

        Ok(Icon {
            id: 0, // TODO: this gets created on db insert
            rid,
            name,
            alias,
            code,
            status,
            search_categories: category,
            category: figma_category,
            tags,
            notes,
            released_at,
            last_updated_at,
            deprecated_at,
            published,
        })
    }
}

fn as_string(value: &prost_types::Value) -> Option<String> {
    match &value.kind {
        Some(prost_types::value::Kind::StringValue(v)) => Some(v.clone()),
        _ => None,
    }
}

fn as_string_array(value: &prost_types::Value) -> Vec<String> {
    match &value.kind {
        Some(prost_types::value::Kind::StringValue(v)) => {
            v.split(", ").map(|s| s.trim().to_string()).collect()
        }
        Some(prost_types::value::Kind::ListValue(v)) => v
            .values
            .iter()
            .filter_map(|v| match &v.kind {
                Some(prost_types::value::Kind::StringValue(s)) => Some(s.clone()),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

fn as_float(value: &prost_types::Value) -> Option<f64> {
    match &value.kind {
        Some(prost_types::value::Kind::NumberValue(v)) => Some(*v as f64),
        Some(prost_types::value::Kind::StringValue(v)) => v.parse::<f64>().ok(),
        _ => None,
    }
}

fn as_int(value: &prost_types::Value) -> Option<i32> {
    match &value.kind {
        Some(prost_types::value::Kind::NumberValue(v)) => Some(*v as i32),
        Some(prost_types::value::Kind::StringValue(v)) => v.parse::<i32>().ok(),
        _ => None,
    }
}

fn as_bool(value: &prost_types::Value) -> bool {
    match &value.kind {
        Some(prost_types::value::Kind::BoolValue(v)) => *v,
        _ => false,
    }
}

fn as_status(value: &prost_types::Value) -> Option<IconStatus> {
    match &value.kind {
        Some(prost_types::value::Kind::StringValue(v)) => serde_plain::from_str(v).ok(),
        _ => None,
    }
}

fn as_category(value: &prost_types::Value) -> Option<FigmaCategory> {
    match &value.kind {
        Some(prost_types::value::Kind::StringValue(v)) => serde_plain::from_str(v).ok(),
        _ => None,
    }
}

fn as_search_categories(value: &prost_types::Value) -> Vec<Category> {
    match &value.kind {
        Some(prost_types::value::Kind::ListValue(v)) => v
            .values
            .iter()
            .filter_map(|v| match &v.kind {
                Some(prost_types::value::Kind::StringValue(s)) => serde_plain::from_str(s).ok(),
                _ => None,
            })
            .collect(),
        _ => Default::default(),
    }
}
