use crate::icons::{Icon, IconCategory, IconStatus};
use serde::{Deserialize, Deserializer};
use sqlx::{migrate::Migrator, PgPool, Pool, Postgres, QueryBuilder};
use std::env;
use std::str::FromStr;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

#[derive(Debug)]
pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    #[tracing::instrument(level = "info")]
    pub async fn init() -> Result<Self, sqlx::Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let pool = PgPool::connect(&database_url).await?;

        MIGRATOR.run(&pool).await?;

        Ok(Database { pool })
    }

    #[tracing::instrument(level = "info")]
    pub async fn get_icons(&self, query: IconQuery) -> Result<Vec<Icon>, sqlx::Error> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new("SELECT * FROM icons");
        let mut has_clauses = false;

        if query.has_clauses() {
            builder.push(" WHERE ");

            if let Some(published) = query.published {
                builder.push("published = ").push_bind(published);
                has_clauses = true;
            }

            if let Some(name) = query.name {
                if has_clauses {
                    builder.push(" AND ");
                }
                builder.push("name = ").push_bind(name);
            }

            if let Some(status) = query.status {
                if !status.is_empty() {
                    if has_clauses {
                        builder.push(" AND ");
                    }
                    builder.push("status IN ");

                    let mut list = builder.separated(", ");
                    list.push_unseparated("(");
                    for status in status {
                        list.push_bind(status.to_string());
                    }
                    list.push_unseparated(")");

                    has_clauses = true;
                }
            }

            if let Some(category) = query.category {
                if !category.is_empty() {
                    if has_clauses {
                        builder.push(" AND ");
                    }

                    builder.push("category IN ");

                    let mut list = builder.separated(", ");
                    list.push_unseparated("(");
                    for category in category {
                        list.push_bind(category.to_string());
                    }
                    list.push_unseparated(")");

                    has_clauses = true;
                }
            }

            if let Some(tags) = query.tags {
                if !tags.is_empty() {
                    if has_clauses {
                        builder.push(" AND ");
                    }
                    builder.push("tags && ").push_bind(tags);
                }
            }

            if let Some(release) = query.released {
                if has_clauses {
                    builder.push(" AND ");
                }
                match release {
                    IconReleaseQuery::Exact(v) => {
                        builder.push("released_at = ").push_bind(v);
                    }
                    IconReleaseQuery::Range(a, b) => {
                        builder
                            .push("released_at BETWEEN ")
                            .push_bind(a)
                            .push(" AND ")
                            .push_bind(b);
                    }
                    IconReleaseQuery::LessThanOrEqual(v) => {
                        builder.push("released_at <= ").push_bind(v);
                    }
                    IconReleaseQuery::GraterThanOrEqual(v) => {
                        builder.push("released_at >= ").push_bind(v);
                    }
                }
            }
        }

        let dir = query.dir.unwrap_or_default();
        match query.order {
            None | Some(OrderColumn::Name) => {
                builder.push(format!(" ORDER BY name {}", dir));
            }
            Some(OrderColumn::Status) => {
                builder.push(format!(" ORDER BY status {}", dir));
            }
            Some(OrderColumn::Release) => {
                builder.push(format!(" ORDER BY released_at {}", dir));
            }
            Some(OrderColumn::Code) => {
                builder.push(format!(" ORDER BY code {}", dir));
            }
        }

        builder.build_query_as::<Icon>().fetch_all(&self.pool).await
    }

    #[tracing::instrument(level = "info")]
    pub async fn get_icon_by_name(&self, name: &str) -> Result<Option<Icon>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM icons WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
    }

    #[tracing::instrument(level = "info")]
    pub async fn upsert_icon(&self, icon: &Icon) -> Result<Icon, sqlx::Error> {
        sqlx::query_as(
            "INSERT INTO icons (rid, name, status, category, search_categories, tags, notes, released_at, last_updated_at, deprecated_at, published, alias, code) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) ON CONFLICT (rid) DO UPDATE SET name = EXCLUDED.name RETURNING *",
        )
        .bind(&icon.rid)
        .bind(&icon.name)
        .bind(icon.status.to_string())
        .bind(icon.category.to_string())
        .bind(icon.search_categories.iter().map(|s| s.to_string()).collect::<Vec<_>>())
        .bind(&icon.tags)
        .bind(&icon.notes)
        .bind(&icon.released_at)
        .bind(&icon.last_updated_at)
        .bind(&icon.deprecated_at)
        .bind(icon.published)
        .bind(&icon.alias)
        .bind(icon.code)
        .fetch_one(&self.pool)
        .await
    }

    #[tracing::instrument(level = "info")]
    pub async fn delete_icon(&self, rid: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM icons WHERE rid = $1")
            .bind(rid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "info")]
    pub async fn get_icon_by_id(&self, id: i32) -> Result<Option<Icon>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM icons WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    #[tracing::instrument(level = "info")]
    pub async fn get_icon_by_rid(&self, rid: &str) -> Result<Option<Icon>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM icons WHERE rid = $1")
            .bind(rid)
            .fetch_optional(&self.pool)
            .await
    }

    #[tracing::instrument(level = "info")]
    pub async fn search_icons(
        &self,
        name: &str,
        status: Option<&str>,
        category: Option<&str>,
    ) -> Result<Vec<Icon>, sqlx::Error> {
        let mut query = "SELECT * FROM icons WHERE name ILIKE $1".to_string();
        let mut params = vec![format!("%{}%", name)];

        if let Some(status) = status {
            query.push_str(" AND status = $2");
            params.push(status.to_string());
        }

        if let Some(category) = category {
            query.push_str(" AND category = $3");
            params.push(category.to_string());
        }

        sqlx::query_as(&query)
            .bind(params)
            .fetch_all(&self.pool)
            .await
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct IconQuery {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_csv")]
    pub status: Option<Vec<IconStatus>>,
    #[serde(default, deserialize_with = "deserialize_csv")]
    pub category: Option<Vec<IconCategory>>,
    #[serde(default, deserialize_with = "deserialize_csv")]
    pub tags: Option<Vec<String>>,
    pub search_categories: Option<Vec<IconCategory>>,
    pub published: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_optional_icon_release_query")]
    pub released: Option<IconReleaseQuery>,
    #[serde(default, deserialize_with = "deserialize_optional_icon_release_query")]
    pub updated: Option<IconReleaseQuery>,
    pub deprecated: Option<bool>,
    pub order: Option<OrderColumn>,
    pub dir: Option<OrderDirection>,
}

impl IconQuery {
    pub fn new() -> Self {
        IconQuery::default().published(true)
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn status(mut self, status: Vec<IconStatus>) -> Self {
        self.status = Some(status);
        self
    }

    pub fn category(mut self, category: Vec<IconCategory>) -> Self {
        self.category = Some(category);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn search_categories(mut self, search_categories: Vec<IconCategory>) -> Self {
        self.search_categories = Some(search_categories);
        self
    }

    pub fn published(mut self, published: bool) -> Self {
        self.published = Some(published);
        self
    }

    pub fn released(mut self, released: IconReleaseQuery) -> Self {
        self.released = Some(released);
        self
    }

    pub fn updated(mut self, updated: IconReleaseQuery) -> Self {
        self.updated = Some(updated);
        self
    }

    pub fn deprecated(mut self, deprecated: bool) -> Self {
        self.deprecated = Some(deprecated);
        self
    }

    pub fn has_clauses(&self) -> bool {
        self.name.is_some()
            || self.status.is_some()
            || self.category.is_some()
            || self.tags.is_some()
            || self.search_categories.is_some()
            || self.published.is_some()
            || self.released.is_some()
            || self.updated.is_some()
            || self.deprecated.is_some()
    }
}

fn deserialize_csv<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: std::fmt::Display,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    let s = s.map(|s| {
        s.split(',')
            .map(str::trim)
            .map(str::parse::<T>)
            .collect::<Result<_, _>>()
    });
    match s {
        Some(Ok(v)) => Ok(Some(v)),
        Some(Err(e)) => Err(serde::de::Error::custom(format!(
            "Failed to parse CSV: {}",
            e
        ))),
        None => Ok(None),
    }
}

#[derive(Debug, Clone)]
pub enum IconReleaseQuery {
    Exact(f64),
    Range(f64, f64),
    LessThanOrEqual(f64),
    GraterThanOrEqual(f64),
}

impl FromStr for IconReleaseQuery {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((a, b)) = s.split_once("..") {
            match (a.trim(), b.trim()) {
                ("", b) => {
                    let b = b
                        .parse::<f64>()
                        .map_err(|e| format!("Invalid number: {}", e))?;
                    Ok(IconReleaseQuery::LessThanOrEqual(b))
                }
                (a, "") => {
                    let a = a
                        .parse::<f64>()
                        .map_err(|e| format!("Invalid number: {}", e))?;
                    Ok(IconReleaseQuery::GraterThanOrEqual(a))
                }
                (a, b) => {
                    let a = a
                        .parse::<f64>()
                        .map_err(|e| format!("Invalid number: {}", e))?;
                    let b = b
                        .parse::<f64>()
                        .map_err(|e| format!("Invalid number: {}", e))?;
                    Ok(IconReleaseQuery::Range(a, b))
                }
            }
        } else {
            let val = s
                .parse::<f64>()
                .map_err(|e| format!("Invalid number: {}", e))?;
            Ok(IconReleaseQuery::Exact(val))
        }
    }
}

fn deserialize_optional_icon_release_query<'de, D>(
    deserializer: D,
) -> Result<Option<IconReleaseQuery>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => IconReleaseQuery::from_str(&s)
            .map(Some)
            .map_err(serde::de::Error::custom),
        None => Ok(None),
    }
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderColumn {
    #[default]
    Name,
    Status,
    Release,
    Code,
}

#[derive(Debug, Default, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderDirection {
    #[default]
    Asc,
    Desc,
}

impl std::fmt::Display for OrderDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderDirection::Asc => write!(f, "ASC"),
            OrderDirection::Desc => write!(f, "DESC"),
        }
    }
}
