#![allow(clippy::needless_return, non_snake_case)]

pub mod services;
pub mod handlers;
pub mod models;

use crate::{services::postgres::PostgresService, handlers::*};
use axum::{
    http::{HeaderValue, Method},
    routing::{get, IntoMakeService},
    Router,
};
use hyper::{header::HeaderName, server::conn::AddrIncoming};
use std::str::FromStr;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
pub struct AppState {
    // Todo: Move this to models
    redis_db: std::sync::Arc<tokio::sync::Mutex<redis::aio::Connection>>,
    databaseService: PostgresService,
}

pub async fn redis_init() -> redis::aio::Connection {
    let redisHost = std::env::var("REDIS_HOST").expect("env::var REDIS_HOST failed");
    let redisClient =
        redis::Client::open(format!("redis://{redisHost}")).expect("Failed to create Redis client");
    let redisConnection: redis::aio::Connection = redisClient
        .get_async_connection()
        .await
        .expect("Failed to get Redis connection");
    return redisConnection;
}

pub fn run(
    listener: std::net::TcpListener,
    redisConnection: redis::aio::Connection,
    databaseService: PostgresService,
) -> Result<hyper::server::Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    let redisDb = std::sync::Arc::new(tokio::sync::Mutex::new(redisConnection));

    let appState = AppState {
        redis_db: redisDb.clone(),
        databaseService,
    };

    let corsOrigins = [
        "http://localhost:3000".parse().unwrap(),
        "http://127.0.0.1:3000".parse().unwrap(),
        "https://unsaferust.org".parse().unwrap(),
        "http://unsaferust.org".parse().unwrap(),
    ];
    let cors = CorsLayer::new()
        .allow_methods([Method::GET])
        .allow_origin(corsOrigins)
        .max_age(std::time::Duration::from_secs(60) * 60);

    // Routes
    let providerRoutes = Router::new()
        .route("/:id", get(getProviderById))
        .route("/", get(getProviders))
        .with_state(appState.clone());
    let providerRoutesNamespace = Router::new().nest("/providers", providerRoutes);

    let projectRoutes = Router::new()
        .route("/import", get(projectsImport))
        .route("/:id", get(getProjectById))
        .route("/", get(getProjects))
        .with_state(appState.clone());
    let projectRoutesNamespace = Router::new().nest("/projects", projectRoutes);

    let projectStatsRoutes: Router<()> = Router::new()
        .route("/update", get(updateProjectsStats))
        .route("/:id", get(getProjectStatsById))
        .route("/", get(getProjectsStats))
        .with_state(appState);
    let projectStatsNamespace = Router::new().nest("/project-stats", projectStatsRoutes);
    let apiV1Namespace = Router::new().nest(
        "/v1",
        providerRoutesNamespace
            .merge(projectRoutesNamespace)
            .merge(projectStatsNamespace),
    );

    let apiNamespaceRoutes = axum::routing::Router::new()
        .route("/health_check", get(healthCheck))
        .route("/redis/flush", get(redisFlush))
        .with_state(redisDb);
    let apiNamespace = Router::new().nest("/api", apiNamespaceRoutes.merge(apiV1Namespace));
    let app = Router::new()
        .merge(apiNamespace)
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

    let server = axum::Server::from_tcp(listener)
        .expect("axum::Server::from_tcp failed")
        .serve(app.into_make_service());

    return Ok(server);
}
