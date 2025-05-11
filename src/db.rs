use crate::icons::Icon;
use sqlx::{migrate::Migrator, PgPool};
use sqlx::{Pool, Postgres};
use std::env;

static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

pub struct Database {
    pool: Pool<Postgres>,
}

impl Database {
    pub async fn init() -> Result<Self, sqlx::Error> {
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL not set");
        let pool = PgPool::connect(&database_url).await?;

        MIGRATOR.run(&pool).await?;

        Ok(Database { pool })
    }

    pub async fn get_icons(&self) -> Result<Vec<Icon>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM icons")
            .fetch_all(&self.pool)
            .await
    }

    pub async fn get_icon_by_name(&self, name: &str) -> Result<Option<Icon>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM icons WHERE name = $1")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
    }

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

    pub async fn delete_icon(&self, rid: &str) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM icons WHERE rid = $1")
            .bind(rid)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn get_icon_by_id(&self, id: i32) -> Result<Option<Icon>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM icons WHERE id = $1")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_icon_by_rid(&self, rid: &str) -> Result<Option<Icon>, sqlx::Error> {
        sqlx::query_as("SELECT * FROM icons WHERE rid = $1")
            .bind(rid)
            .fetch_optional(&self.pool)
            .await
    }

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
