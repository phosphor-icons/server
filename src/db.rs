use crate::entities::{icons, svgs};
use crate::icons::{Category, IconStatus, LibraryInfo};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    prelude::*, Condition, Database, DatabaseConnection, Order, QueryOrder, QuerySelect,
};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;
use utoipa::{IntoParams, ToSchema};

#[derive(Debug)]
pub struct Db {
    pub conn: DatabaseConnection,
}

impl Db {
    #[tracing::instrument(level = "info")]
    pub async fn init() -> Result<Self, sea_orm::DbErr> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let conn = Database::connect(database_url).await?;
        Ok(Self { conn })
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn ping(&self) -> Result<(), DbErr> {
        self.conn.ping().await
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn dump_stats(&self) -> Result<(), ()> {
        tracing::info!("Conn: {:?}", self.conn);
        Ok(())
    }

    #[tracing::instrument(level = "info")]
    fn build_condition_from_params(query: &IconQuery) -> Condition {
        let mut cond = Condition::all();

        if let Some(name) = &query.name {
            let comp = if name.starts_with('*') || name.ends_with('*') {
                let trimmed = name.trim_matches('*');
                if trimmed.is_empty() {
                    return cond; // If the name is just '*', return empty condition
                }
                icons::Column::Name.like(format!("%{}%", trimmed))
            } else {
                icons::Column::Name.eq(name)
            };
            cond = cond.add(comp);
        }

        match &query.published {
            Some(Ternary::True) | None => cond = cond.add(icons::Column::Published.eq(true)),
            Some(Ternary::False) => cond = cond.add(icons::Column::Published.eq(false)),
            Some(Ternary::Any) => {}
        }

        if let Some(released) = &query.released {
            match released {
                IconReleaseQuery::Exact(v) => {
                    cond = cond.add(icons::Column::ReleasedAt.eq(*v));
                }
                IconReleaseQuery::Range(a, b) => {
                    cond = cond.add(icons::Column::ReleasedAt.between(*a, *b));
                }
                IconReleaseQuery::LessThanOrEqual(v) => {
                    cond = cond.add(icons::Column::ReleasedAt.lte(*v));
                }
                IconReleaseQuery::GraterThanOrEqual(v) => {
                    cond = cond.add(icons::Column::ReleasedAt.gte(*v));
                }
            }
        }

        if let Some(updated) = &query.updated {
            match updated {
                IconReleaseQuery::Exact(v) => {
                    cond = cond.add(icons::Column::LastUpdatedAt.eq(*v));
                }
                IconReleaseQuery::Range(a, b) => {
                    cond = cond.add(icons::Column::LastUpdatedAt.between(*a, *b));
                }
                IconReleaseQuery::LessThanOrEqual(v) => {
                    cond = cond.add(icons::Column::LastUpdatedAt.lte(*v));
                }
                IconReleaseQuery::GraterThanOrEqual(v) => {
                    cond = cond.add(icons::Column::LastUpdatedAt.gte(*v));
                }
            }
        }

        if let Some(deprecated) = &query.deprecated {
            match deprecated {
                IconReleaseQuery::Exact(v) => {
                    cond = cond.add(icons::Column::DeprecatedAt.eq(*v));
                }
                IconReleaseQuery::Range(a, b) => {
                    cond = cond.add(icons::Column::DeprecatedAt.between(*a, *b));
                }
                IconReleaseQuery::LessThanOrEqual(v) => {
                    cond = cond.add(icons::Column::DeprecatedAt.lte(*v));
                }
                IconReleaseQuery::GraterThanOrEqual(v) => {
                    cond = cond.add(icons::Column::DeprecatedAt.gte(*v));
                }
            }
        }

        if let Some(status) = &query.status {
            cond = cond.add(icons::Column::Status.is_in(status.iter().map(|s| s.to_string())));
        }

        if let Some(category) = &query.category {
            cond = cond.add(Expr::cust_with_values(
                "search_categories && $1",
                [category.iter().map(|c| c.to_string()).collect::<Vec<_>>()],
            ));
        }

        if let Some(tags) = &query.tags {
            cond = cond.add(Expr::cust_with_values("tags && $1", [tags.clone()]));
        }

        cond
    }

    #[tracing::instrument(level = "info")]
    fn build_order_from_params(query: &IconQuery) -> (icons::Column, Order) {
        let order_column = match query.order {
            None | Some(OrderColumn::Name) => icons::Column::Name,
            Some(OrderColumn::Status) => icons::Column::Status,
            Some(OrderColumn::Release) => icons::Column::ReleasedAt,
            Some(OrderColumn::Code) => icons::Column::Code,
        };

        let order_direction = match query.dir {
            Some(OrderDirection::Asc) | None => Order::Asc,
            Some(OrderDirection::Desc) => Order::Desc,
        };

        (order_column, order_direction)
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get_icons(&self, query: &IconQuery) -> Result<Vec<icons::Model>, DbErr> {
        let cond = Self::build_condition_from_params(query);
        let (ord, dir) = Self::build_order_from_params(query);
        icons::Entity::find()
            .filter(cond)
            .order_by(ord, dir)
            .all(&self.conn)
            .await
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn count_icons(&self, query: &IconQuery) -> Result<u64, DbErr> {
        let cond = Self::build_condition_from_params(query);
        icons::Entity::find().filter(cond).count(&self.conn).await
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get_icon_by_name(&self, name: &str) -> Result<Option<icons::Model>, DbErr> {
        icons::Entity::find()
            .filter(icons::Column::Name.eq(name))
            .one(&self.conn)
            .await
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get_icon_by_id(&self, id: i32) -> Result<Option<icons::Model>, DbErr> {
        icons::Entity::find()
            .filter(icons::Column::Id.eq(id))
            .one(&self.conn)
            .await
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get_icon_by_rid(&self, rid: &str) -> Result<Option<icons::Model>, DbErr> {
        icons::Entity::find()
            .filter(icons::Column::Rid.eq(rid))
            .one(&self.conn)
            .await
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get_icon_by_code(&self, code: i32) -> Result<Option<icons::Model>, DbErr> {
        icons::Entity::find()
            .filter(icons::Column::Code.eq(code))
            .one(&self.conn)
            .await
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn upsert_icon(&self, icon: icons::Model) -> Result<i32, DbErr> {
        let active_model: icons::ActiveModel = icon.into();
        let res = icons::Entity::insert(active_model)
            .on_conflict(
                OnConflict::column(icons::Column::Rid)
                    .update_column(icons::Column::Name)
                    .update_column(icons::Column::Status)
                    .update_column(icons::Column::Category)
                    .update_column(icons::Column::SearchCategories)
                    .update_column(icons::Column::Tags)
                    .update_column(icons::Column::Notes)
                    .update_column(icons::Column::ReleasedAt)
                    .update_column(icons::Column::LastUpdatedAt)
                    .update_column(icons::Column::DeprecatedAt)
                    .update_column(icons::Column::Published)
                    .update_column(icons::Column::Alias)
                    .update_column(icons::Column::Code)
                    .to_owned(),
            )
            .exec(&self.conn)
            .await?;
        Ok(res.last_insert_id)
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn delete_icon(&self, rid: &str) -> Result<u64, DbErr> {
        icons::Entity::delete_many()
            .filter(icons::Column::Rid.eq(rid))
            .exec(&self.conn)
            .await
            .map(|res| res.rows_affected)
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn query_icons(&self, query: &IconSearch) -> Result<Vec<icons::Model>, DbErr> {
        todo!("Implement query_icons with fuzzy search and relevance");
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get_all_tags(&self) -> Result<Vec<String>, DbErr> {
        icons::Entity::find()
            .select_only()
            .column(icons::Column::Tags)
            .all(&self.conn)
            .await
            .map(|models| {
                models
                    .into_iter()
                    .flat_map(|model| model.tags)
                    .collect::<Vec<_>>()
            })
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get_icon_weights_by_icon_id(
        &self,
        icon_id: i32,
    ) -> Result<HashMap<String, svgs::Model>, DbErr> {
        let svgs: Vec<svgs::Model> = svgs::Entity::find()
            .filter(svgs::Column::IconId.eq(icon_id))
            .all(&self.conn)
            .await?;

        Ok(svgs
            .into_iter()
            .map(|s| (s.weight.clone(), s))
            .collect::<HashMap<_, _>>())
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn upsert_svg(&self, svg: svgs::Model) -> Result<i32, DbErr> {
        let active_model: svgs::ActiveModel = svg.into();
        let res = svgs::Entity::insert(active_model)
            .on_conflict(
                OnConflict::columns(vec![svgs::Column::IconId, svgs::Column::Weight])
                    .update_column(svgs::Column::Src)
                    .to_owned(),
            )
            .exec(&self.conn)
            .await?;
        Ok(res.last_insert_id)
    }

    #[tracing::instrument(level = "info", skip(self))]
    pub async fn get_library_info(&self) -> Result<LibraryInfo, DbErr> {
        icons::Entity::find()
            .select_only()
            .column_as(Expr::col(icons::Column::Id).count(), "count")
            .column_as(Expr::col(icons::Column::ReleasedAt).max(), "version")
            .filter(icons::Column::Published.eq(true))
            .into_model::<LibraryInfo>()
            .one(&self.conn)
            .await
            .map(|opt| {
                opt.unwrap_or_else(|| LibraryInfo {
                    count: 0,
                    version: 0.0,
                })
            })
    }
}

#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query, style = Form)]
pub struct IconSearch {
    /// A fuzzy search term to match against icon names, aliases, tags, and othjer properties.
    #[serde(alias = "query")]
    #[param(example = "block")]
    pub q: String,
}

#[derive(Debug, Default, Deserialize, IntoParams)]
#[into_params(parameter_in = Query, style = Form)]
pub struct IconQuery {
    /// Filter search results by kebab-case icon name. Supports wildcards (`*`) at the beginning and/or end of expression.
    pub name: Option<String>,
    /// Filter search results by version or version ranges in which they were published, including exact
    /// versions (`2.1`), open-ended inclusive ranges (`..1.4` or `2.0..`), and closed inclusive
    /// ranges (`1.5..2.0`). All versions are in the format `<major>.<minor>`.
    #[serde(
        default,
        rename = "v",
        alias = "released",
        deserialize_with = "deserialize_optional_icon_release_query"
    )]
    #[param(example = "1.5..2.0")]
    pub released: Option<IconReleaseQuery>,
    /// Filter search results by whether the icon is published. When `true` (default), only icons
    /// that are currently available are returned. When `false`, only icons that are incomplete or
    /// removed are returned. When `any`, results are not filtered by published state.
    #[param(example = "any")]
    pub published: Option<Ternary>,
    #[serde(
        skip,
        default,
        deserialize_with = "deserialize_optional_icon_release_query"
    )]
    pub updated: Option<IconReleaseQuery>,
    #[serde(skip)]
    pub deprecated: Option<IconReleaseQuery>,
    /// Filter search results by one or more comma-separated release statuses.
    #[serde(default, deserialize_with = "deserialize_csv")]
    #[param(explode = false)]
    pub status: Option<Vec<IconStatus>>,
    /// Filter search results by one or more comma-separated icon categories.
    #[serde(default, deserialize_with = "deserialize_csv")]
    #[param(explode = false)]
    pub category: Option<Vec<Category>>,
    /// Filter search results by one or more comma-separated tags.
    #[serde(default, deserialize_with = "deserialize_csv")]
    #[param(explode = false)]
    pub tags: Option<Vec<String>>,
    pub order: Option<OrderColumn>,
    pub dir: Option<OrderDirection>,
}

impl IconQuery {
    pub fn new() -> Self {
        IconQuery::default().published(Ternary::True)
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn status(mut self, status: Vec<IconStatus>) -> Self {
        self.status = Some(status);
        self
    }

    pub fn category(mut self, category: Vec<Category>) -> Self {
        self.category = Some(category);
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    pub fn published(mut self, published: Ternary) -> Self {
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

    pub fn deprecated(mut self, deprecated: IconReleaseQuery) -> Self {
        self.deprecated = Some(deprecated);
        self
    }

    pub fn has_clauses(&self) -> bool {
        self.name.is_some()
            || self.status.is_some()
            || self.category.is_some()
            || self.tags.is_some()
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

#[derive(Debug, Clone, ToSchema)]
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

#[derive(Debug, Default, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum OrderColumn {
    #[default]
    Name,
    Status,
    Release,
    Code,
}

#[derive(Debug, Default, Clone, Copy, Deserialize, ToSchema)]
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

#[derive(Debug, Default, Clone, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Ternary {
    #[default]
    True,
    False,
    Any,
}
