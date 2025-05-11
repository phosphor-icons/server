use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};
use phosphor_server::db::Database;
use phosphor_server::table::TableClient;
use std::sync::Mutex;
use utoipa::ToSchema;

struct AppState {
    client: Mutex<TableClient>,
    db: Mutex<Database>,
}

#[get("/")]
async fn all_icons(data: web::Data<AppState>) -> impl Responder {
    let db = data.db.lock().unwrap();
    match db.get_icons().await {
        Ok(icons) => HttpResponse::Ok().json(icons),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[post("/search")]
async fn search_icons(data: web::Data<AppState>, query: web::Json<String>) -> impl Responder {
    let db = data.db.lock().unwrap();
    match db.get_icon_by_name(&query).await {
        Ok(Some(icon)) => HttpResponse::Ok().json(icon),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();

    let table_client = TableClient::init().await.map_err(|_| {
        eprintln!("Failed to initialize table client");
        std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to initialize table client",
        )
    })?;

    let db = Database::init().await.map_err(|_| {
        eprintln!("Failed to initialize database");
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to initialize database")
    })?;

    let icons = table_client.sync().await.map_err(|_| {
        eprintln!("Failed to sync table client");
        std::io::Error::new(std::io::ErrorKind::Other, "Failed to sync table client")
    })?;

    for icon in icons {
        db.upsert_icon(&icon).await.map_err(|_| {
            eprintln!("Failed to upsert icon");
            std::io::Error::new(std::io::ErrorKind::Other, "Failed to upsert icon")
        })?;
    }

    let data = web::Data::new(AppState {
        client: Mutex::new(table_client),
        db: Mutex::new(db),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(web::scope("/v1").service(all_icons).service(search_icons))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
