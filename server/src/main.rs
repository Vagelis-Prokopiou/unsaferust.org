// Todo: Add home page with info
// Todo: Add error handling
// Todo: Add logging
// Todo: Use redis. Use request uris as keys
// Todo: Check why update is blocking. Comment the db stuff and check performance.
// Todo: Add redis caching
// Todo: Add import url for new projects
// Todo: Fix pagination
// Todo: Fix search
// Todo: Use client caching too
// Todo: Fix footer in mobile
// Todo: Add most popular packages

use std::io::BufRead;
use std::net::TcpListener;
use sqlx::postgres::PgPoolOptions;
use unsaferust::models::configuration::DatabaseSettings;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // let a=1;
    // Prepare the variables that the run method needs.
    let server_port = std::env::var("SERVER_PORT").expect("env::var SERVER_PORT failed");
    let db_user = std::env::var("DB_USER").expect("env::var DB_USER failed");
    let db_password = std::env::var("DB_PASSWORD").expect("env::var DB_PASSWORD failed");
    let db_host = std::env::var("DB_HOST").expect("env::var DB_HOST failed");
    let db_name = std::env::var("DB_NAME").expect("env::var DB_NAME failed");
    let db_port = std::env::var("DB_PORT")
        .unwrap_or_else(|_| "10".to_owned())
        .parse()
        .expect("Failed to parse DB_PORT");
    let db_max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_owned())
        .parse()
        .expect("Failed to parse DB_MAX_CONNECTIONS");
    let db_settings = DatabaseSettings::new(
        db_user,
        db_password,
        db_port,
        db_host,
        db_name,
        db_max_connections,
    );
    let db = PgPoolOptions::new()
        .max_connections(db_max_connections)
        .connect(&db_settings.get_connection_string_with_db())
        .await
        .expect("PgPoolOptions initialization failed");

    // Execute the migrations.
    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("migrations failed");


    // Todo: Delete when done =========================================================================
    let file = std::fs::File::open("./data/providers.txt").expect("Failed to read providers.txt file");
    for url in std::io::BufReader::new(file).lines().flatten() {
        if url.is_empty() { continue; }
        sqlx::query(&format!("
                do
                $do$
                begin
                if not exists (select id from providers where url = '{url}') then
                    insert into providers (url) VALUES ('{url}');
                end if;
                end
                $do$
    "))
            .execute(&db)
            .await
            .expect("Failed to insert to providers");
    }


    // Reading line by line
    let file = std::fs::File::open("./data/projects.txt").expect("Failed to read projects.txt file");
    for line in std::io::BufReader::new(file).lines().flatten() {
        if line.is_empty() { continue; }
        let parts: Vec<&str> = line.split('/').collect();
        if parts.len() != 5 {
            eprint!("Problem with line: {line}");
            continue;
        }

        let provider_url = format!("{}//{}", parts[0], parts[2]);
        let namespace = parts[3];
        let name = parts[4];
        sqlx::query(&format!("
                do
                $do$
                begin
                if not exists (select id from projects where name = '{name}') then
                    insert into projects (provider_id, namespace, name)
                     VALUES (
                     (select id from providers where url = '{provider_url}'),
                     '{namespace}',
                     '{name}'
                     );
                end if;
                end
                $do$
    "))
            .execute(&db)
            .await
            .expect("Failed to insert to projects");
    }
    // Todo: Delete when done =========================================================================


    let address = format!("0.0.0.0:{}", server_port);
    println!("Listening at: {}", &address);
    let listener = TcpListener::bind(&address).expect("TcpListener failed");
    unsaferust::run(listener, db)?.await
}
