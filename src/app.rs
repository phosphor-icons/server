use crate::{db, icons, svgs, table};
use std::sync::Mutex;
use tokio::fs;

#[derive(Debug)]
pub struct AppState {
    pub db: Mutex<db::Db>,
}

impl AppState {
    #[tracing::instrument(level = "info")]
    pub async fn init() -> Result<Self, std::io::Error> {
        let db = db::Db::init().await.map_err(|_| {
            tracing::error!("Failed to initialize database");
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize database")
        })?;

        let mut app = AppState { db: Mutex::new(db) };

        if let Ok(val) = std::env::var("PHOSPHOR_TABLE_SYNC") {
            tracing::info!("PHOSPHOR_TABLE_SYNC={}", val);
            if val == "true" {
                app.sync_table().await?;
            }
        }

        if let Ok(val) = std::env::var("PHOSPHOR_ASSETS_SYNC") {
            tracing::info!("PHOSPHOR_ASSETS_SYNC={}", val);
            if val == "true" {
                app.sync_assets().await?;
            }
        }

        Ok(app)
    }

    #[tracing::instrument(level = "info")]
    async fn sync_table(&mut self) -> Result<(), std::io::Error> {
        tracing::info!("Syncing table client");

        let icons = table::TableClient::sync().await.map_err(|_| {
            tracing::error!("Failed to sync table client");
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to sync table client")
        })?;
        let db = self.db.lock().unwrap();
        for icon in icons {
            db.upsert_icon(icon.clone().into()).await.map_err(|e| {
                tracing::error!("Failed to upsert icon: {:?}: {:?}", &icon, e);
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to upsert icon")
            })?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "info")]
    async fn sync_assets(&self) -> Result<(), std::io::Error> {
        const ASSETS_DIR: &str = "./core/assets";
        tracing::info!("Syncing assets");

        let mut files: Vec<(String, icons::IconWeight)> = Vec::new();

        for weight in icons::IconWeight::ALL {
            let path = format!("{}/{}", ASSETS_DIR, weight.to_string());
            let mut dir = fs::read_dir(&path).await?;

            while let Some(entry) = dir.next_entry().await? {
                if entry.file_type().await?.is_file() {
                    let file_name = entry.file_name().to_str().unwrap().to_owned();
                    if file_name.ends_with(".svg") {
                        let path = format!("{}/{}", path, file_name);
                        files.push((path, weight.clone()));
                    }
                }
            }
        }

        for (path, weight) in files {
            if let Ok(contents) = fs::read_to_string(&path).await {
                let name = path
                    .split('/')
                    .last()
                    .unwrap()
                    .replace("-duotone.svg", "")
                    .replace("-fill.svg", "")
                    .replace("-thin.svg", "")
                    .replace("-light.svg", "")
                    .replace("-bold.svg", "")
                    .replace(".svg", "")
                    .to_string();
                let db = self.db.lock().unwrap();
                if let Some(icon) = db.get_icon_by_name(&name).await.unwrap() {
                    let svg = svgs::Svg {
                        id: 0,
                        icon_id: icon.id,
                        weight: weight.clone(),
                        src: contents,
                    };
                    db.upsert_svg(svg.clone().into()).await.unwrap();
                    tracing::info!("Upserted SVG: {} - {:?}", name, weight);
                } else {
                    tracing::warn!("Icon not found in database: {}", name);
                }
            }
        }

        Ok(())
    }
}
