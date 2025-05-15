use std::str::FromStr;

use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, prelude::*};

use crate::icons::IconWeight;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Svg {
    pub id: i32,
    pub icon_id: i32,
    pub weight: IconWeight,
    pub src: String,
}

impl FromRow<'_, PgRow> for Svg {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        Ok(Svg {
            id: row.try_get("id")?,
            icon_id: row.try_get("icon_id")?,
            weight: row
                .try_get::<String, _>("weight")
                .map(|w| IconWeight::from_str(&w).unwrap_or_default())?,
            src: row.try_get("src")?,
        })
    }
}
