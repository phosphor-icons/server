use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, FromRow, Row};
use std::{fmt::Display, str::FromStr};
use utoipa::ToSchema;

#[derive(Debug, Default, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub struct Icon {
    /// The unique ID of the icon in the database.
    #[serde(default)]
    #[schema(example = 2884)]
    pub id: i32,

    #[serde(rename = "Row ID")]
    #[schema(example = "96cR4kqjHO16pBVCiXg_Ep")]
    pub rid: String,

    /// The kebab-case name of the icon.
    #[schema(example = "cube")]
    pub name: String,

    /// An optional kebab-case alias for the icon, usually a deprecated name.
    #[serde(deserialize_with = "deserialize_string_or_none")]
    pub alias: Option<String>,

    /// The decimal representation of an icon's unicode codepoint, as implemented in
    /// [@phosphor-icons/web](https://github.com/phosphor-icons/web) and other font-based
    /// libraries.
    #[serde(
        rename = "Codepoint",
        deserialize_with = "deserialize_string_to_opt_i32"
    )]
    #[schema(example = 57818)]
    pub code: Option<i32>,

    /// The implementation status of the icon in the design process.
    #[schema(example = "Implemented")]
    pub status: IconStatus,

    /// A list of categories the icon belongs to, used for filtering in the API.
    #[serde(
        rename = "Search Categories",
        deserialize_with = "deserialize_categories"
    )]
    #[schema(example = json!(["Design", "Games", "Objects"]))]
    pub search_categories: Vec<Category>,

    /// A string representing the category the icon belongs to in the Figma library, not used for
    /// filtering in the API.
    pub category: FigmaCategory,

    /// A list of string tags associated with the icon.
    #[serde(deserialize_with = "deserialize_string_array")]
    #[schema(example = json!(["square", "box", "3d", "volume", "blocks"]))]
    pub tags: Vec<String>,

    #[serde(deserialize_with = "deserialize_string_or_none")]
    pub notes: Option<String>,

    /// A float in the format `<major>.<minor>` representing the version in which the icon was
    /// released.
    #[serde(rename = "Release", deserialize_with = "deserialize_string_to_opt_f64")]
    #[schema(example = 1.0)]
    pub released_at: Option<f64>,

    /// A float in the format `<major>.<minor>` representing the version in which the icon was last
    /// updated.
    #[serde(
        rename = "Last Updated",
        deserialize_with = "deserialize_string_to_opt_f64"
    )]
    pub last_updated_at: Option<f64>,

    /// A float in the format `<major>.<minor>` representing the version in which the icon was
    /// deprecated.
    #[serde(
        rename = "Deprecated",
        deserialize_with = "deserialize_string_to_opt_f64"
    )]
    pub deprecated_at: Option<f64>,

    /// A boolean indicating whether the icon is published in the library.
    #[serde(deserialize_with = "deserialize_stringbool")]
    #[schema(example = true)]
    pub published: bool,
}

impl FromRow<'_, PgRow> for Icon {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;
        let rid: String = row.try_get("rid")?;
        let name: String = row.try_get("name")?;

        let status: String = row.try_get("status")?;
        let status = IconStatus::from_str(&status).unwrap_or(IconStatus::None);

        let figma_category: String = row.try_get("category")?;
        let figma_category =
            FigmaCategory::from_str(&figma_category).unwrap_or(FigmaCategory::Unknown);

        let category: Vec<String> = row.try_get("search_categories")?;
        let category: Vec<Category> = category
            .into_iter()
            .map(|s| Category::from_str(&s).unwrap_or(Category::Unknown))
            .collect();

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

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IconWeight {
    Thin,
    Light,
    #[default]
    Regular,
    Bold,
    Fill,
    Duotone,
}

impl IconWeight {
    pub const COUNT: usize = 6;
    pub const ALL: [IconWeight; IconWeight::COUNT] = [
        IconWeight::Thin,
        IconWeight::Light,
        IconWeight::Regular,
        IconWeight::Bold,
        IconWeight::Fill,
        IconWeight::Duotone,
    ];
}

impl Display for IconWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IconWeight::Thin => write!(f, "thin"),
            IconWeight::Light => write!(f, "light"),
            IconWeight::Regular => write!(f, "regular"),
            IconWeight::Bold => write!(f, "bold"),
            IconWeight::Fill => write!(f, "fill"),
            IconWeight::Duotone => write!(f, "duotone"),
        }
    }
}

impl FromStr for IconWeight {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "thin" => Ok(IconWeight::Thin),
            "light" => Ok(IconWeight::Light),
            "regular" => Ok(IconWeight::Regular),
            "bold" => Ok(IconWeight::Bold),
            "fill" => Ok(IconWeight::Fill),
            "duotone" => Ok(IconWeight::Duotone),
            _ => Err(format!("Invalid IconWeight: {}", value)),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash, ToSchema)]
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

impl Display for IconStatus {
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

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum FigmaCategory {
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

impl FigmaCategory {
    pub const COUNT: usize = 19;
    pub const ALL: [FigmaCategory; FigmaCategory::COUNT] = [
        FigmaCategory::Arrows,
        FigmaCategory::Brands,
        FigmaCategory::Commerce,
        FigmaCategory::Communication,
        FigmaCategory::Design,
        FigmaCategory::Development,
        FigmaCategory::Education,
        FigmaCategory::Games,
        FigmaCategory::HealthAndWellness,
        FigmaCategory::MapsAndTravel,
        FigmaCategory::MathAndFinance,
        FigmaCategory::Media,
        FigmaCategory::OfficeAndEditing,
        FigmaCategory::People,
        FigmaCategory::SecurityAndWarnings,
        FigmaCategory::SystemAndDevices,
        FigmaCategory::Time,
        FigmaCategory::WeatherAndNature,
        FigmaCategory::Unknown,
    ];
}

impl FromStr for FigmaCategory {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let res = match value {
            "Arrows" => FigmaCategory::Arrows,
            "Brands" => FigmaCategory::Brands,
            "Commerce" => FigmaCategory::Commerce,
            "Communication" => FigmaCategory::Communication,
            "Design" => FigmaCategory::Design,
            "Development" => FigmaCategory::Development,
            "Education" => FigmaCategory::Education,
            "Games" => FigmaCategory::Games,
            "Health & Wellness" => FigmaCategory::HealthAndWellness,
            "Maps & Travel" => FigmaCategory::MapsAndTravel,
            "Math & Finance" => FigmaCategory::MathAndFinance,
            "Media" => FigmaCategory::Media,
            "Office & Editing" => FigmaCategory::OfficeAndEditing,
            "People" => FigmaCategory::People,
            "Security & Warnings" => FigmaCategory::SecurityAndWarnings,
            "System & Devices" => FigmaCategory::SystemAndDevices,
            "Time" => FigmaCategory::Time,
            "Weather & Nature" => FigmaCategory::WeatherAndNature,
            _ => FigmaCategory::Unknown,
        };
        Ok(res)
    }
}

impl Display for FigmaCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FigmaCategory::Arrows => write!(f, "Arrows"),
            FigmaCategory::Brands => write!(f, "Brands"),
            FigmaCategory::Commerce => write!(f, "Commerce"),
            FigmaCategory::Communication => write!(f, "Communication"),
            FigmaCategory::Design => write!(f, "Design"),
            FigmaCategory::Development => write!(f, "Development"),
            FigmaCategory::Education => write!(f, "Education"),
            FigmaCategory::Games => write!(f, "Games"),
            FigmaCategory::HealthAndWellness => write!(f, "Health & Wellness"),
            FigmaCategory::MapsAndTravel => write!(f, "Maps & Travel"),
            FigmaCategory::MathAndFinance => write!(f, "Math & Finance"),
            FigmaCategory::Media => write!(f, "Media"),
            FigmaCategory::OfficeAndEditing => write!(f, "Office & Editing"),
            FigmaCategory::People => write!(f, "People"),
            FigmaCategory::SecurityAndWarnings => write!(f, "Security & Warnings"),
            FigmaCategory::SystemAndDevices => write!(f, "System & Devices"),
            FigmaCategory::Time => write!(f, "Time"),
            FigmaCategory::WeatherAndNature => write!(f, "Weather & Nature"),
            FigmaCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Hash, ToSchema)]
#[serde(rename_all = "PascalCase")]
pub enum Category {
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

impl Category {
    pub const COUNT: usize = 19;
    pub const ALL: [Category; Category::COUNT] = [
        Category::Arrows,
        Category::Brand,
        Category::Commerce,
        Category::Communication,
        Category::Design,
        Category::Development,
        Category::Editor,
        Category::Finance,
        Category::Games,
        Category::Office,
        Category::Health,
        Category::Map,
        Category::Media,
        Category::Nature,
        Category::Objects,
        Category::People,
        Category::System,
        Category::Weather,
        Category::Unknown,
    ];
}

impl FromStr for Category {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let res = match value {
            "Arrows" => Category::Arrows,
            "Brand" => Category::Brand,
            "Commerce" => Category::Commerce,
            "Communication" => Category::Communication,
            "Design" => Category::Design,
            "Development" => Category::Development,
            "Editor" => Category::Editor,
            "Finance" => Category::Finance,
            "Games" => Category::Games,
            "Office" => Category::Office,
            "Health" => Category::Health,
            "Map" => Category::Map,
            "Media" => Category::Media,
            "Nature" => Category::Nature,
            "Objects" => Category::Objects,
            "People" => Category::People,
            "System" => Category::System,
            "Weather" => Category::Weather,
            _ => Category::Unknown,
        };
        Ok(res)
    }
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

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Category::Arrows => write!(f, "Arrows"),
            Category::Brand => write!(f, "Brand"),
            Category::Commerce => write!(f, "Commerce"),
            Category::Communication => write!(f, "Communication"),
            Category::Design => write!(f, "Design"),
            Category::Development => write!(f, "Development"),
            Category::Editor => write!(f, "Editor"),
            Category::Finance => write!(f, "Finance"),
            Category::Games => write!(f, "Games"),
            Category::Office => write!(f, "Office"),
            Category::Health => write!(f, "Health"),
            Category::Map => write!(f, "Map"),
            Category::Media => write!(f, "Media"),
            Category::Nature => write!(f, "Nature"),
            Category::Objects => write!(f, "Objects"),
            Category::People => write!(f, "People"),
            Category::System => write!(f, "System"),
            Category::Weather => write!(f, "Weather"),
            Category::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LibraryInfo {
    /// The current version of the library.
    #[schema(example = 2.1f64)]
    pub version: f64,
    /// The total number of published icons as of the current version.
    #[schema(example = 1512)]
    pub count: usize,
}

impl FromRow<'_, PgRow> for LibraryInfo {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let version: f64 = row.try_get("version")?;
        let count: i64 = row.try_get("count")?;

        Ok(LibraryInfo {
            version,
            count: count as usize,
        })
    }
}
