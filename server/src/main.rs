#![allow(non_snake_case)]

// Todo: Add cascade on delete to projects.
// Todo: Check why update is blocking. Comment the services stuff and check performance.
// Todo: Use client caching too
// Todo: Add most popular packages
// Todo: Add structs for code_lines: i32 and unsafe_lines: i32.

use std::io::BufRead;
use std::net::TcpListener;
use unsaferust::{services::postgres::PostgresService};
use unsaferust::services::redis::RedisService;

#[tokio::main]
async fn main() {
    // Prepare the variables that the run method needs.
    let serverPort = std::env::var("SERVER_PORT").expect("env::var SERVER_PORT failed");
    let databaseService = PostgresService::new(None).await;
    let _redisService = RedisService::new().await;

    // Execute the migrations.
    sqlx::migrate!("./migrations")
        .run(&databaseService.connection)
        .await
        .expect("migrations failed");

    // Todo: Delete when done =========================================================================
    let file =
        std::fs::File::open("./data/providers.txt").expect("Failed to read providers.txt file");
    for url in std::io::BufReader::new(file).lines().flatten() {
        if url.is_empty() {
            continue;
        }
        sqlx::query(&format!(
            "
                do
                $do$
                begin
                if not exists (select id from providers where url = '{url}') then
                    insert into providers (url) VALUES ('{url}');
                end if;
                end
                $do$
    "
        ))
            .execute(&databaseService.connection)
            .await
            .expect("Failed to insert to providers");
    }

    let file =
        std::fs::File::open("./data/projects.txt").expect("Failed to read projects.txt file");
    for project in std::io::BufReader::new(file).lines().flatten() {
        if project.is_empty() {
            continue;
        }
        let project = project.replace("https://", "");
        let parts: Vec<&str> = project.split('/').collect();
        let namespace = parts[1];
        let name = parts[2];
        println!("parts: {:?}", parts);
        sqlx::query(&format!(
            "
                    do
                    $do$
                    begin
                    if not exists (select id from projects where namespace = '{namespace}' and name = '{name}') then
                        insert into projects (provider_id, namespace, name) VALUES ('1', '{namespace}', '{name}');
                    end if;
                    end
                    $do$
        "
        ))
            .execute(&databaseService.connection)
            .await
            .expect("Failed to insert to providers");
    }
    // Todo: Delete when done =========================================================================

    let address = format!("0.0.0.0:{}", serverPort);
    let listener = TcpListener::bind(&address).expect("TcpListener failed");
    println!("Listening at: {}", &address);
    unsaferust::run(listener, unsaferust::redis_init().await, databaseService)
        .expect("unsaferust::run failed")
        .await
        .expect("axum::Server failed");
}
