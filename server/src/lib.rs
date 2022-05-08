// Place the following at the top of lib.rs (or main.rs) for global effect.
#![allow(clippy::needless_return)]

pub mod handlers;
pub mod models;

use crate::handlers::*;
use actix_web::dev::Server;
use actix_web::web::Data;
use actix_web::{App, HttpServer, web};
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
