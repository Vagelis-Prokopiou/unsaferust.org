#![allow(clippy::needless_return, non_snake_case)]

pub mod handlers;
pub mod models;

use crate::handlers::*;
use axum::http::{HeaderValue, Method};
use axum::{
    routing::{get, IntoMakeService},
    Router,
};
use hyper::header::HeaderName;
use hyper::server::conn::AddrIncoming;
use sqlx::PgPool;
use std::str::FromStr;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    // Todo: Move this to models
    redis_db: std::sync::Arc<tokio::sync::Mutex<redis::aio::Connection>>,
    postgress_db: std::sync::Arc<PgPool>,
}

pub async fn redis_init() -> redis::aio::Connection {
    let redis_host = std::env::var("REDIS_HOST").expect("env::var REDIS_HOST failed");
    let redis_client = redis::Client::open(format!("redis://{redis_host}"))
        .expect("Failed to create Redis client");
    let redis_connection: redis::aio::Connection = redis_client
        .get_async_connection()
        .await
        .expect("Failed to get Redis connection");
    return redis_connection;
}

pub fn run(
    listener: std::net::TcpListener,
    db: PgPool,
    redis_connection: redis::aio::Connection,
) -> Result<hyper::server::Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    let redis_db = std::sync::Arc::new(tokio::sync::Mutex::new(redis_connection));
    let postgress_db = std::sync::Arc::new(db);
    let appState = AppState {
        redis_db: redis_db.clone(),
        postgress_db: postgress_db.clone(),
    };

    let cors_origins = [
        "http://localhost:3000".parse().unwrap(),
        "http://127.0.0.1:3000".parse().unwrap(),
        "https://unsaferust.org".parse().unwrap(),
        "http://unsaferust.org".parse().unwrap(),
    ];
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(cors_origins)
        .max_age(std::time::Duration::from_secs(60) * 60);

    // Routes
    let provider_routes = Router::new()
        .route("/:id", get(providers_get_by_id))
        .route("/", get(providers_get_all))
        .with_state(postgress_db.clone());
    let provider_routes_namespace = Router::new().nest("/providers", provider_routes);

    let project_routes = Router::new()
        .route("/import", get(projects_import))
        .route("/:id", get(projects_get_by_id))
        .route("/", get(projects_get_all))
        .with_state(postgress_db);
    let project_routes_namespace = Router::new().nest("/projects", project_routes);

    let project_stats_routes: Router<()> = Router::new()
        .route("/update", get(project_stats_update))
        .route("/:id", get(project_stats_get_by_id))
        .route("/", get(project_stats_get_all))
        .with_state(appState);

    let project_stats_namespace = Router::new().nest("/project-stats", project_stats_routes);

    let api_v1_namespace = Router::new().nest(
        "/v1",
        provider_routes_namespace
            .merge(project_routes_namespace)
            .merge(project_stats_namespace),
    );

    let api_namespace_routes = axum::routing::Router::new()
        .route("/health_check", get(health_check))
        .route("/redis/flush", get(redis_flush))
        .with_state(redis_db);
    let api_namespace = Router::new().nest("/api", api_namespace_routes.merge(api_v1_namespace));
    let app = Router::new()
        .merge(api_namespace)
        .layer(cors)
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("X-Content-Type-Options").unwrap(),
            HeaderValue::from_str("nosniff").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("X-Content-Type-Options").unwrap(),
            HeaderValue::from_str("nosniff").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("X-Frame-Options").unwrap(),
            HeaderValue::from_str("DENY").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("X-XSS-Protection").unwrap(),
            HeaderValue::from_str("0").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("Strict-Transport-Security").unwrap(),
            HeaderValue::from_str("max-age=63072000; includeSubDomains; preload").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("Content-Security-Policy").unwrap(),
            HeaderValue::from_str("default-src https:").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("Referrer-Policy").unwrap(),
            HeaderValue::from_str("strict-origin-when-cross-origin").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("Referrer-Policy").unwrap(),
            HeaderValue::from_str("strict-origin-when-cross-origin").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("Cross-Origin-Resource-Policy").unwrap(),
            HeaderValue::from_str("same-site").unwrap(),
        ))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(
            HeaderName::from_str("Cross-Origin-Embedder-Policy").unwrap(),
            HeaderValue::from_str("require-corp").unwrap(),
        ))
        .layer(
            tower_http::set_header::SetResponseHeaderLayer::if_not_present(
                HeaderName::from_str("Content-Type").unwrap(),
                HeaderValue::from_str("text/html; charset=UTF-8").unwrap(),
            ),
        );
    //         .wrap(DefaultHeaders::new().add(("Access-Control-Allow-Origin", "https://unsaferust.org")))

    let server = axum::Server::from_tcp(listener)
        .expect("axum::Server::from_tcp failed")
        .serve(app.into_make_service());

    return Ok(server);
}
