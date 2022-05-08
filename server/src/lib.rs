// Place the following at the top of lib.rs (or main.rs) for global effect.
#![allow(clippy::needless_return)]

pub mod handlers;
pub mod models;

use crate::handlers::*;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
use actix_web::middleware::DefaultHeaders;
use sqlx::PgPool;


pub fn run(
    listener: std::net::TcpListener,
    db: PgPool,
) -> Result<Server, std::io::Error> {
    let db = Data::new(db);
    let server = HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://127.0.0.1:3000")
            .allowed_origin("https://unsaferust.org")
            .allowed_origin("http://unsaferust.org")
            .allowed_methods(vec!["GET"])
            .max_age(3600);


        App::new()
            .wrap(DefaultHeaders::new().add(("Strict-Transport-Security", "max-age=63072000; includeSubDomains; preload")))
            .wrap(DefaultHeaders::new().add(("Content-Security-Policy", "default-src https:")))
            .wrap(DefaultHeaders::new().add(("X-XSS-Protection", "0")))
            .wrap(DefaultHeaders::new().add(("X-Frame-Options", "DENY")))
            .wrap(DefaultHeaders::new().add(("X-Content-Type-Options", "nosniff")))
            .wrap(DefaultHeaders::new().add(("Referrer-Policy", "strict-origin-when-cross-origin")))
            .wrap(DefaultHeaders::new().add(("Content-Type", "text/html; charset=UTF-8")))
            .wrap(DefaultHeaders::new().add(("Access-Control-Allow-Origin", "https://unsaferust.org")))
            .wrap(DefaultHeaders::new().add(("Cross-Origin-Resource-Policy", "same-site")))
            .wrap(DefaultHeaders::new().add(("Cross-Origin-Embedder-Policy", "require-corp")))
            .wrap(cors)
            .app_data(db.clone())
            .route("/", web::get().to(index))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/v1")
                            .route("/health_check", web::get().to(health_check))
                            .service(
                                web::scope("/providers")
                                    .route("/{id}", web::get().to(providers_get_by_id))
                                    .route("", web::get().to(providers_get_all))
                            )
                            .service(
                                web::scope("/projects")
                                    .route("/{id}", web::get().to(projects_get_by_id))
                                    .route("", web::get().to(projects_get_all))
                            )
                            .service(
                                web::scope("/project-stats")
                                    .route("/update", web::get().to(project_stats_update))
                                    .route("/{id}", web::get().to(project_stats_get_by_id))
                                    .route("", web::get().to(project_stats_get_all))
                            )
                    )
            )
    })
        .listen(listener)?
        .run();
    Ok(server)
}
