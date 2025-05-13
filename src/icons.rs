use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};

#[derive(Debug, Default, Serialize)]
pub struct Icon {
    pub id: i32,
    pub rid: String,
    pub name: String,
    pub status: IconStatus,
    pub category: IconCategory,
    pub search_categories: Vec<IconSearchCategory>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub released_at: Option<f64>,
    pub last_updated_at: Option<f64>,
    pub deprecated_at: Option<f64>,
    pub published: bool,
    pub alias: Option<String>,
    pub code: Option<i32>,
}

impl FromRow<'_, PgRow> for Icon {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;
        let rid: String = row.try_get("rid")?;
        let name: String = row.try_get("name")?;

        let status: String = row.try_get("status")?;
        let status = IconStatus::from_str(&status).unwrap_or(IconStatus::None);

        let category: String = row.try_get("category")?;
        let category = IconCategory::from_str(&category).unwrap_or(IconCategory::Unknown);

        let search_categories: Vec<String> = row.try_get("search_categories")?;
        let search_categories: Vec<IconSearchCategory> =
            search_categories.into_iter().map(|s| s.into()).collect();

        let tags: Vec<String> = row.try_get("tags")?;
        let notes: Option<String> = row.try_get("notes")?;
        let released_at: Option<f64> = row.try_get("released_at")?;
        let last_updated_at: Option<f64> = row.try_get("last_updated_at")?;
        let deprecated_at: Option<f64> = row.try_get("deprecated_at")?;
        let published: bool = row.try_get("published")?;
        let alias: Option<String> = row.try_get("alias")?;
        let code: Option<i32> = row.try_get("code")?;

        Ok(Icon {
            id,
            rid,
            name,
            status,
            category,
            search_categories,
            tags,
            notes,
            released_at,
            last_updated_at,
            deprecated_at,
            published,
            alias,
            code,
        })
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum IconStatus {
    Backlog,
    Designing,
    Designed,
    Implemented,
    Deprecated,
    #[default]
    #[serde(other)]
    None,
}

impl IconStatus {
    pub const COUNT: usize = 6;
    pub const ALL: [IconStatus; IconStatus::COUNT] = [
        IconStatus::Backlog,
        IconStatus::Designing,
        IconStatus::Designed,
        IconStatus::Implemented,
        IconStatus::Deprecated,
        IconStatus::None,
    ];
}

impl FromStr for IconStatus {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Backlog" => Ok(IconStatus::Backlog),
            "Designing" => Ok(IconStatus::Designing),
            "Designed" => Ok(IconStatus::Designed),
            "Implemented" => Ok(IconStatus::Implemented),
            "Deprecated" => Ok(IconStatus::Deprecated),
            _ => Ok(IconStatus::None),
        }
    }
}

impl std::fmt::Display for IconStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconStatus::Backlog => write!(f, "Backlog"),
            IconStatus::Designing => write!(f, "Designing"),
            IconStatus::Designed => write!(f, "Designed"),
            IconStatus::Implemented => write!(f, "Implemented"),
            IconStatus::Deprecated => write!(f, "Deprecated"),
            IconStatus::None => write!(f, "None"),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum IconCategory {
    Arrows,
    Brands,
    Commerce,
    Communication,
    Design,
    Development,
    Education,
    Games,
    #[serde(rename = "Health & Wellness")]
    HealthAndWellness,
    #[serde(rename = "Maps & Travel")]
    MapsAndTravel,
    #[serde(rename = "Math & Finance")]
    MathAndFinance,
    Media,
    #[serde(rename = "Office & Editing")]
    OfficeAndEditing,
    People,
    #[serde(rename = "Security & Warnings")]
    SecurityAndWarnings,
    #[serde(rename = "System & Devices")]
    SystemAndDevices,
    Time,
    #[serde(rename = "Weather & Nature")]
    WeatherAndNature,
    #[default]
    #[serde(other)]
    Unknown,
}

impl IconCategory {
    pub const COUNT: usize = 19;
    pub const ALL: [IconCategory; IconCategory::COUNT] = [
        IconCategory::Arrows,
        IconCategory::Brands,
        IconCategory::Commerce,
        IconCategory::Communication,
        IconCategory::Design,
        IconCategory::Development,
        IconCategory::Education,
        IconCategory::Games,
        IconCategory::HealthAndWellness,
        IconCategory::MapsAndTravel,
        IconCategory::MathAndFinance,
        IconCategory::Media,
        IconCategory::OfficeAndEditing,
        IconCategory::People,
        IconCategory::SecurityAndWarnings,
        IconCategory::SystemAndDevices,
        IconCategory::Time,
        IconCategory::WeatherAndNature,
        IconCategory::Unknown,
    ];
}

impl FromStr for IconCategory {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let res = match value {
            "Arrows" => IconCategory::Arrows,
            "Brands" => IconCategory::Brands,
            "Commerce" => IconCategory::Commerce,
            "Communication" => IconCategory::Communication,
            "Design" => IconCategory::Design,
            "Development" => IconCategory::Development,
            "Education" => IconCategory::Education,
            "Games" => IconCategory::Games,
            "Health & Wellness" => IconCategory::HealthAndWellness,
            "Maps & Travel" => IconCategory::MapsAndTravel,
            "Math & Finance" => IconCategory::MathAndFinance,
            "Media" => IconCategory::Media,
            "Office & Editing" => IconCategory::OfficeAndEditing,
            "People" => IconCategory::People,
            "Security & Warnings" => IconCategory::SecurityAndWarnings,
            "System & Devices" => IconCategory::SystemAndDevices,
            "Time" => IconCategory::Time,
            "Weather & Nature" => IconCategory::WeatherAndNature,
            _ => IconCategory::Unknown,
        };
        Ok(res)
    }
}

impl std::fmt::Display for IconCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconCategory::Arrows => write!(f, "Arrows"),
            IconCategory::Brands => write!(f, "Brands"),
            IconCategory::Commerce => write!(f, "Commerce"),
            IconCategory::Communication => write!(f, "Communication"),
            IconCategory::Design => write!(f, "Design"),
            IconCategory::Development => write!(f, "Development"),
            IconCategory::Education => write!(f, "Education"),
            IconCategory::Games => write!(f, "Games"),
            IconCategory::HealthAndWellness => write!(f, "Health & Wellness"),
            IconCategory::MapsAndTravel => write!(f, "Maps & Travel"),
            IconCategory::MathAndFinance => write!(f, "Math & Finance"),
            IconCategory::Media => write!(f, "Media"),
            IconCategory::OfficeAndEditing => write!(f, "Office & Editing"),
            IconCategory::People => write!(f, "People"),
            IconCategory::SecurityAndWarnings => write!(f, "Security & Warnings"),
            IconCategory::SystemAndDevices => write!(f, "System & Devices"),
            IconCategory::Time => write!(f, "Time"),
            IconCategory::WeatherAndNature => write!(f, "Weather & Nature"),
            IconCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "PascalCase")]
pub enum IconSearchCategory {
    Arrows,
    Brand,
    Commerce,
    Communication,
    Design,
    Development,
    Editor,
    Finance,
    Games,
    Office,
    Health,
    Map,
    Media,
    Nature,
    Objects,
    People,
    System,
    Weather,
    #[serde(other)]
    Unknown,
}

impl IconSearchCategory {
    pub const COUNT: usize = 19;
    pub const ALL: [IconSearchCategory; IconSearchCategory::COUNT] = [
        IconSearchCategory::Arrows,
        IconSearchCategory::Brand,
        IconSearchCategory::Commerce,
        IconSearchCategory::Communication,
        IconSearchCategory::Design,
        IconSearchCategory::Development,
        IconSearchCategory::Editor,
        IconSearchCategory::Finance,
        IconSearchCategory::Games,
        IconSearchCategory::Office,
        IconSearchCategory::Health,
        IconSearchCategory::Map,
        IconSearchCategory::Media,
        IconSearchCategory::Nature,
        IconSearchCategory::Objects,
        IconSearchCategory::People,
        IconSearchCategory::System,
        IconSearchCategory::Weather,
        IconSearchCategory::Unknown,
    ];
}

impl From<String> for IconSearchCategory {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Arrows" => IconSearchCategory::Arrows,
            "Brand" => IconSearchCategory::Brand,
            "Commerce" => IconSearchCategory::Commerce,
            "Communication" => IconSearchCategory::Communication,
            "Design" => IconSearchCategory::Design,
            "Development" => IconSearchCategory::Development,
            "Editor" => IconSearchCategory::Editor,
            "Finance" => IconSearchCategory::Finance,
            "Games" => IconSearchCategory::Games,
            "Office" => IconSearchCategory::Office,
            "Health" => IconSearchCategory::Health,
            "Map" => IconSearchCategory::Map,
            "Media" => IconSearchCategory::Media,
            "Nature" => IconSearchCategory::Nature,
            "Objects" => IconSearchCategory::Objects,
            "People" => IconSearchCategory::People,
            "System" => IconSearchCategory::System,
            "Weather" => IconSearchCategory::Weather,
            _ => IconSearchCategory::Unknown,
        }
    }
}

impl std::fmt::Display for IconSearchCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconSearchCategory::Arrows => write!(f, "Arrows"),
            IconSearchCategory::Brand => write!(f, "Brand"),
            IconSearchCategory::Commerce => write!(f, "Commerce"),
            IconSearchCategory::Communication => write!(f, "Communication"),
            IconSearchCategory::Design => write!(f, "Design"),
            IconSearchCategory::Development => write!(f, "Development"),
            IconSearchCategory::Editor => write!(f, "Editor"),
            IconSearchCategory::Finance => write!(f, "Finance"),
            IconSearchCategory::Games => write!(f, "Games"),
            IconSearchCategory::Office => write!(f, "Office"),
            IconSearchCategory::Health => write!(f, "Health"),
            IconSearchCategory::Map => write!(f, "Map"),
            IconSearchCategory::Media => write!(f, "Media"),
            IconSearchCategory::Nature => write!(f, "Nature"),
            IconSearchCategory::Objects => write!(f, "Objects"),
            IconSearchCategory::People => write!(f, "People"),
            IconSearchCategory::System => write!(f, "System"),
            IconSearchCategory::Weather => write!(f, "Weather"),
            IconSearchCategory::Unknown => write!(f, "Unknown"),
        }
    }
}
