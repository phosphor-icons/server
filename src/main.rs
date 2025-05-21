use std::{net::Ipv4Addr, time::Duration};

use actix_web::{get, http, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use phosphor_server::app;
use serde::Serialize;
use tracing_subscriber::{filter::EnvFilter, prelude::*};
use utoipa::{self, OpenApi};
use utoipa_actix_web::{scope, AppExt};
use utoipa_scalar::{Scalar, Servable as ScalarServable};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "Phosphor Icons API",
        description = include_str!("../public/intro.md"),
        version = "0.1.0",
        contact(name = "Phosphor Team", email = "hello@phosphoricons.com"),
        license(name = "MIT", identifier = "MIT"),
    ),
    tags(
        (
            name = "Icon endpoints",
            description = "Search and filter existing, deprecated, and upcoming icons, and retrieve SVG source code for specific icons."
        ),
        (name = "Metadata endpoints", description = "Query for metadata about the API, including available categories and tags."),
        (name = "Other endpoints", description = "Other endpoints"),
    ),
)]
struct Api;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    dotenvy::dotenv().ok();

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
                    .service(icons::icon)
                    .service(icons::all_icons)
                    .service(icons::search_icons)
                    .service(categories::categories)
                    .service(tags::tags),
            )
            .service(health::health_check)
            .openapi_service(|api| {
                let api = Api::openapi().merge_from(api);
                Scalar::with_url("/docs", api).custom_html(include_str!("../public/index.html"))
            })
            .into_app()
            .service(health::dump)
            .service(actix_files::Files::new("/", "./public"))
    })
    // NOTE: the app requires a minimum of 3 workers to run the docs server, dispatch, and at
    // least one request handler. We should look at real-world utilization once this is public.
    .workers(8)
    .keep_alive(Duration::from_secs(120))
    .bind((url, port))?
    .run()
    .await
}

mod icons {
    use super::*;
    use phosphor_server::{app, db, icons, svgs};
    use serde_qs::actix::QsQuery;
    use std::collections::HashMap;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    pub struct IconWeightMap {
        #[schema(example = "<svg>...</svg>")]
        regular: String,
        #[schema(example = "<svg>...</svg>")]
        thin: String,
        #[schema(example = "<svg>...</svg>")]
        light: String,
        #[schema(example = "<svg>...</svg>")]
        bold: String,
        #[schema(example = "<svg>...</svg>")]
        fill: String,
        #[schema(example = "<svg>...</svg>")]
        duotone: String,
    }

    impl From<HashMap<icons::IconWeight, svgs::Svg>> for IconWeightMap {
        fn from(map: HashMap<icons::IconWeight, svgs::Svg>) -> Self {
            Self {
                regular: map
                    .get(&icons::IconWeight::Regular)
                    .map(|s| s.src.clone())
                    .unwrap_or_default(),
                thin: map
                    .get(&icons::IconWeight::Thin)
                    .map(|s| s.src.clone())
                    .unwrap_or_default(),
                light: map
                    .get(&icons::IconWeight::Light)
                    .map(|s| s.src.clone())
                    .unwrap_or_default(),
                bold: map
                    .get(&icons::IconWeight::Bold)
                    .map(|s| s.src.clone())
                    .unwrap_or_default(),
                fill: map
                    .get(&icons::IconWeight::Fill)
                    .map(|s| s.src.clone())
                    .unwrap_or_default(),
                duotone: map
                    .get(&icons::IconWeight::Duotone)
                    .map(|s| s.src.clone())
                    .unwrap_or_default(),
            }
        }
    }

    #[derive(ToSchema, Serialize)]
    pub struct SingleIconResponse {
        /// Icon metadata
        icon: icons::Icon,
        /// SVG code for the icon
        svgs: IconWeightMap,
    }

    #[utoipa::path(
        description = "Fetch an icon by its ID, returning the icon's metadata and SVG code.",
        params(
            ("id", example = 2884),
        ),
        responses(
            (status = OK, body = SingleIconResponse, description = "Icon found"),
            (status = NOT_FOUND, description = "Icon not found"),
            (status = INTERNAL_SERVER_ERROR, description = "Internal server error"),
        ),
        tag = "Icon endpoints",
    )]
    #[get("/icon/{id}")]
    #[tracing::instrument(level = "info")]
    async fn icon(data: web::Data<app::AppState>, id: web::Path<i32>) -> impl Responder {
        let db = data.db.lock().unwrap();
        let id = id.into_inner();
        dbg!(id);
        match db.get_icon_by_id(id).await {
            Ok(Some(icon)) => {
                if let Ok(svgmap) = db.get_svg_weights_by_icon_id(id).await {
                    let svgs = IconWeightMap::from(svgmap);
                    HttpResponse::Ok().json(SingleIconResponse { icon, svgs })
                } else {
                    tracing::error!("Failed to fetch SVGs for icon: {}", id);
                    HttpResponse::InternalServerError().finish()
                }
            }
            Ok(None) => {
                tracing::info!("Icon not found: {}", id);
                HttpResponse::NotFound().finish()
            }
            Err(_) => {
                tracing::error!("Failed to fetch icons");
                HttpResponse::InternalServerError().finish()
            }
        }
    }

    #[derive(ToSchema, Serialize)]
    pub struct MultipleIconResponse {
        icons: Vec<icons::Icon>,
        count: usize,
    }

    impl MultipleIconResponse {
        pub fn new(icons: Vec<icons::Icon>) -> Self {
            let count = icons.len();
            Self { icons, count }
        }
    }

    #[utoipa::path(
        description = "Fetch icons from our database, with optional query parameters to filter by name, status, release version, tags, and categories.",
        params(db::IconQuery),
        responses(
            (status = OK, body = MultipleIconResponse),
            (status = INTERNAL_SERVER_ERROR, description = "Internal server error"),
        ),
        tag = "Icon endpoints",
    )]
    #[get("/icons")]
    #[tracing::instrument(level = "info")]
    async fn all_icons(
        data: web::Data<app::AppState>,
        query: QsQuery<db::IconQuery>,
    ) -> impl Responder {
        let db = data.db.lock().unwrap();
        let query = query.into_inner();
        match db.get_icons(&query).await {
            Ok(icons) => HttpResponse::Ok()
                .insert_header((http::header::ACCESS_CONTROL_ALLOW_ORIGIN, "*"))
                .json(MultipleIconResponse::new(icons)),
            Err(e) => {
                tracing::error!("Failed to fetch icons for query: {:?}", e);
                HttpResponse::InternalServerError().finish()
            }
        }
    }

    #[utoipa::path(
        description = "Fuzzy search for icons by semantic name, use-case, or other properties. Returns results along with a relevance score.",
        params(db::IconSearch),
        responses(
            (status = OK, body = MultipleIconResponse),
            (status = NOT_FOUND, description = "Icon not found"),
            (status = INTERNAL_SERVER_ERROR, description = "Internal server error"),
        ),
        tag = "Icon endpoints",
    )]
    #[get("/search")]
    #[tracing::instrument(level = "info")]
    async fn search_icons(
        data: web::Data<app::AppState>,
        search: web::Query<db::IconSearch>,
    ) -> impl Responder {
        let db = data.db.lock().unwrap();
        let search = search.into_inner();
        match db.fuzzy_search_icons(&search).await {
            Ok(icons) => HttpResponse::Ok().json(MultipleIconResponse::new(icons)),
            Err(_) => {
                tracing::error!("Failed to fetch icon: {:?}", search);
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
    struct CategoriesResponse {
        categories: Vec<icons::Category>,
        count: usize,
    }

    #[utoipa::path(
        description = "Fetch all icon categories from our database. These can be used as the `category` parameter in the [/v1/icons](#tag/icon-endpoints/GET/v1/icons) endpoint.",
        responses((status = OK, body = CategoriesResponse)),
        tag = "Metadata endpoints",

    )]
    #[get("/categories")]
    #[tracing::instrument(level = "info")]
    async fn categories() -> impl Responder {
        HttpResponse::Ok().json(CategoriesResponse {
            categories: icons::Category::ALL.to_vec(),
            count: icons::Category::COUNT,
        })
    }
}

mod tags {
    use super::*;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    struct TagsResponse {
        tags: Vec<String>,
        count: usize,
    }

    #[utoipa::path(
        description = "Fetch all unique icon tags from our database. These can be used as the `tags` parameter in the [/v1/icons](#tag/default/GET/v1/icons) endpoint.",
        responses(
            (status = OK, body = TagsResponse),
            (status = INTERNAL_SERVER_ERROR, description = "Internal server error"),
        ),
        tag = "Metadata endpoints",
    )]
    #[get("/tags")]
    #[tracing::instrument(level = "info")]
    async fn tags(data: web::Data<app::AppState>) -> impl Responder {
        let db = data.db.lock().unwrap();
        match db.get_all_tags().await {
            Ok(tags) => {
                let count = tags.len();
                HttpResponse::Ok().json(TagsResponse { tags, count })
            }
            Err(_) => {
                tracing::error!("Failed to fetch tags");
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}

mod health {
    use super::*;
    use utoipa::ToSchema;

    #[derive(Serialize, ToSchema)]
    #[serde(rename_all = "snake_case")]
    enum HealthStatus {
        Healthy,
        Degraded,
        Down,
    }

    #[derive(Serialize, ToSchema)]
    struct HealthResponse {
        status: HealthStatus,
    }

    #[utoipa::path(
        description = "Reports the health of the API. Returns `healthy` if the database is reachable, `degraded` if there are issues, and `down` if the database is unreachable.",
        responses(
            (
                status = OK,
                body = HealthResponse,
                description = "Service is healthy",
            ),
            (
                status = SERVICE_UNAVAILABLE,
                body = HealthResponse,
                example = json!(HealthResponse { status: HealthStatus::Down }),,
                description = "Service is down, unreachable",
            ),
            (
                status = INTERNAL_SERVER_ERROR,
                body = HealthResponse,
                example = json!(HealthResponse { status: HealthStatus::Degraded }),,
                description = "Service is degraded, connected but unresponsive",
            ),
        ),
        tag = "Other endpoints",
    )]
    #[get("/health")]
    #[tracing::instrument(level = "info")]
    async fn health_check(data: web::Data<app::AppState>) -> impl Responder {
        match data.db.lock() {
            Ok(db) => {
                if let Err(e) = db.ping().await {
                    tracing::error!("Database ping failed: {e}");
                    return HttpResponse::InternalServerError().json(HealthResponse {
                        status: HealthStatus::Degraded,
                    });
                }

                HttpResponse::Ok().json(HealthResponse {
                    status: HealthStatus::Healthy,
                })
            }
            Err(e) => {
                tracing::error!("Failed to acquire database lock: {e}");
                HttpResponse::ServiceUnavailable().json(HealthResponse {
                    status: HealthStatus::Down,
                })
            }
        }
    }

    #[get("/dump")]
    #[tracing::instrument(level = "info")]
    pub async fn dump(data: web::Data<app::AppState>) -> impl Responder {
        let db = data.db.lock().unwrap();
        match db.dump_stats().await {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(e) => {
                tracing::error!("Failed to dump database: {e}");
                HttpResponse::InternalServerError().finish()
            }
        }
    }
}
