#![allow(clippy::needless_return, non_snake_case)]

pub mod handlers;
pub mod models;
pub mod services;
mod utils;

use crate::{
    handlers::*,
    models::appState::AppState,
    services::{postgres::PostgresService, redis::RedisService},
};
use axum::{
    http::{HeaderValue, Method},
    routing::{get, IntoMakeService},
    Router,
};
use hyper::{header::HeaderName, server::conn::AddrIncoming};
use std::str::FromStr;
use tower_http::cors::CorsLayer;

pub fn run(
    listener: std::net::TcpListener,
    redisService: RedisService,
    databaseService: PostgresService,
) -> Result<hyper::server::Server<AddrIncoming, IntoMakeService<Router>>, std::io::Error> {
    let appState = AppState {
        redisService,
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
        .with_state(appState.clone());
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
        .with_state(appState);
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
