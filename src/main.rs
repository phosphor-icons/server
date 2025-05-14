use std::net::Ipv4Addr;

use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use phosphor_server::app;
use serde::Serialize;
use serde_qs::actix::QsQuery;
use tracing_subscriber::{filter::EnvFilter, prelude::*};
use utoipa;
use utoipa_actix_web::{scope, AppExt};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

mod icons {
    use super::*;
    use phosphor_server::{app, db, icons};
    use utoipa::ToSchema;

    #[derive(ToSchema, Serialize)]
    pub struct MultipleIconsResponse {
        icons: Vec<icons::Icon>,
        count: usize,
    }

    impl MultipleIconsResponse {
        pub fn new(icons: Vec<icons::Icon>) -> Self {
            let count = icons.len();
            Self { icons, count }
        }
    }

    #[utoipa::path(
        responses((status = OK, body = icons::Icon)),
        params(db::IconQuery)
    )]
    #[get("/all")]
    #[tracing::instrument(level = "info")]
    async fn all_icons(
        data: web::Data<app::AppState>,
        query: QsQuery<db::IconQuery>,
    ) -> impl Responder {
        let db = data.db.lock().unwrap();
        match db.get_icons(query.into_inner()).await {
            Ok(icons) => HttpResponse::Ok().json(MultipleIconsResponse::new(icons)),
            Err(_) => {
                tracing::error!("Failed to fetch icons for query");
                HttpResponse::InternalServerError().finish()
            }
        }
    }

    #[utoipa::path(responses((status = OK, body = icons::Icon)))]
    #[get("/search")]
    #[tracing::instrument(level = "info")]
    async fn search_icons(
        data: web::Data<app::AppState>,
        query: web::Json<String>,
    ) -> impl Responder {
        let db = data.db.lock().unwrap();
        match db.get_icon_by_name(&query).await {
            Ok(Some(icon)) => HttpResponse::Ok().json(icon),
            Ok(None) => {
                tracing::info!("Icon not found: {}", query);
                HttpResponse::NotFound().finish()
            }
            Err(_) => {
                tracing::error!("Failed to fetch icon: {}", query);
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

mod categories {
    use super::*;
    use phosphor_server::icons;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    struct Response {
        categories: Vec<icons::Category>,
        count: usize,
    }

    #[utoipa::path(responses((status = OK, body = icons::Icon)))]
    #[get("/categories")]
    #[tracing::instrument(level = "info")]
    async fn categories() -> impl Responder {
        HttpResponse::Ok().json(Response {
            categories: icons::Category::ALL.to_vec(),
            count: icons::Category::COUNT,
        })
    }
}

mod health {
    use super::*;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    struct Response {
        status: String,
    }

    #[utoipa::path(responses((status = OK, body = Response)))]
    #[get("/health")]
    #[tracing::instrument(level = "info")]
    async fn health_check(data: web::Data<app::AppState>) -> impl Responder {
        let db = data.db.lock().unwrap();
        if let Err(_) = db.ping().await {
            tracing::error!("Database connection failed");
            return HttpResponse::InternalServerError().finish();
        }

        HttpResponse::Ok().json(Response {
            status: "ok".to_string(),
        })
    }
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();

    let app = app::AppState::init().await?;
    let data = web::Data::new(app);
    let url = std::env::var("HOST").unwrap_or(Ipv4Addr::UNSPECIFIED.to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");

    HttpServer::new(move || {
        App::new()
            .into_utoipa_app()
            .app_data(data.clone())
            .map(|app| app.wrap(Logger::default()))
            .service(
                scope::scope("/v1")
                    .service(icons::all_icons)
                    .service(icons::search_icons)
                    .service(categories::categories)
                    .service(health::health_check),
            )
            .openapi_service(|api| Scalar::with_url("/scalar", api))
            .into_app()
    })
    // .workers(4)
    .bind((url, port))?
    .run()
    .await
}
