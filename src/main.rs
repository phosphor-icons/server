use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use phosphor_server::{app, db::IconQuery, icons::IconCategory};
use serde::Serialize;
use serde_qs::actix::QsQuery;
use tracing_subscriber::{filter::EnvFilter, prelude::*};
use utoipa::ToSchema;

#[get("/all")]
#[tracing::instrument(level = "info")]
async fn all_icons(data: web::Data<app::AppState>, query: QsQuery<IconQuery>) -> impl Responder {
    let db = data.db.lock().unwrap();
    match db.get_icons(query.into_inner()).await {
        Ok(icons) => HttpResponse::Ok().json(app::MultipleIconsResponse::new(icons)),
        Err(_) => {
            tracing::error!("Failed to fetch icons for query");
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[get("/search")]
#[tracing::instrument(level = "info")]
async fn search_icons(data: web::Data<app::AppState>, query: web::Json<String>) -> impl Responder {
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

#[get("/categories")]
#[tracing::instrument(level = "info")]
async fn categories() -> impl Responder {
    #[derive(Serialize, ToSchema)]
    struct Response {
        categories: Vec<IconCategory>,
        count: usize,
    }

    HttpResponse::Ok().json(Response {
        categories: IconCategory::ALL.to_vec(),
        count: IconCategory::COUNT,
    })
}

#[get("/health")]
#[tracing::instrument(level = "info")]
async fn health_check(data: web::Data<app::AppState>) -> impl Responder {
    #[derive(Serialize, ToSchema)]
    struct Response {
        status: String,
    }

    let db = data.db.lock().unwrap();
    if let Err(_) = db.ping().await {
        tracing::error!("Database connection failed");
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().json(Response {
        status: "ok".to_string(),
    })
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
    let url = std::env::var("HOST").expect("HOST must be set");
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a valid u16");

    HttpServer::new(move || {
        App::new().app_data(data.clone()).service(
            web::scope("/v1")
                .service(all_icons)
                .service(search_icons)
                .service(categories)
                .service(health_check),
        )
    })
    .workers(4)
    .bind((url, port))?
    .run()
    .await
}
