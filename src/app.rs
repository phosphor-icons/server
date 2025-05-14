use crate::{db, table};
use std::sync::Mutex;

#[derive(Debug)]
pub struct AppState {
    pub client: Mutex<table::TableClient>,
    pub db: Mutex<db::Database>,
}

impl AppState {
    #[tracing::instrument(level = "info")]
    pub async fn init() -> Result<Self, std::io::Error> {
        let table_client = table::TableClient::init().await.map_err(|_| {
            tracing::error!("Failed to initialize table client");
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to initialize table client",
            )
        })?;
        let db = db::Database::init().await.map_err(|_| {
            tracing::error!("Failed to initialize database");
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize database")
        })?;

        let mut app = AppState {
            client: Mutex::new(table_client),
            db: Mutex::new(db),
        };

        if let Ok(val) = std::env::var("PHOSPHOR_SERVER_SYNC") {
            tracing::info!("PHOSPHOR_SERVER_SYNC={}", val);
            if val == "true" {
                app.sync().await?;
            }
        }

        Ok(app)
    }

    #[tracing::instrument(level = "info")]
    async fn sync(&mut self) -> Result<(), std::io::Error> {
        println!("Syncing table client");
        let client = self.client.lock().unwrap();
        let icons = client.sync().await.map_err(|_| {
            tracing::error!("Failed to sync table client");
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to sync table client")
        })?;

        let db = self.db.lock().unwrap();
        for icon in icons {
            db.upsert_icon(&icon).await.map_err(|e| {
                tracing::error!("Failed to upsert icon: {:?}", e);
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to upsert icon")
            })?;
        }

        Ok(())
    }
}
