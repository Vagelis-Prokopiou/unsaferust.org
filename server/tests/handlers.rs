use serde_json::Value;
use sqlx::Executor;
use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;
use unsaferust::models::configuration::DatabaseSettings;
use unsaferust::models::project::{Project, ProjectStats};
use unsaferust::models::provider::Provider;
use uuid::Uuid;

lazy_static::lazy_static! { static ref CLIENT: reqwest::Client = reqwest::Client::new(); }

#[tokio::test]
async fn test_health_check() {
    let (address, _db) = spawn_app().await;
    let response = CLIENT
        .get(format!("{}/api/v1/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
}

async fn spawn_app() -> (String, PgPool) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    // We retrieve the port assigned to us by the OS
    let port = listener.local_addr().unwrap().port();
    let db_user = std::env::var("DB_USER").expect("env::var DB_USER failed");
    let db_password = std::env::var("DB_PASSWORD").expect("env::var DB_PASSWORD failed");
    let db_host = std::env::var("DB_HOST").expect("env::var DB_HOST failed");
    let db_port = std::env::var("DB_PORT")
        .unwrap_or_else(|_| "10".to_owned())
        .parse()
        .expect("Failed to parse DB_PORT");
    let db_max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_owned())
        .parse()
        .expect("Failed to parse DB_MAX_CONNECTIONS");
    let db_name = Uuid::new_v4().to_string();

    let db_settings = DatabaseSettings::new(
        db_user,
        db_password,
        db_port,
        db_host,
        db_name,
        db_max_connections,
    );
    let connection_pool = configure_database(&db_settings).await;

    let server =
        unsaferust::run(listener, connection_pool.clone()).expect("Failed to bind address");
    let _ = tokio::spawn(server);
    // We return the application address to the caller!
    return (format!("http://127.0.0.1:{}", port), connection_pool);
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create database
    let mut connection = PgConnection::connect(&config.get_connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    let connection_pool = PgPool::connect(&config.get_connection_string_with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");
    return connection_pool;
}

async fn create_provider(db: &PgPool) -> bool {
    let result = sqlx::query("insert into providers (url) values ( 'https://github.com')")
        .execute(db)
        .await
        .expect("Failed to create entry");
    return result.rows_affected() == 2;
}

async fn create_project(db: &PgPool) -> bool {
    let result = sqlx::query(
        "
        insert into projects (provider_id, name, namespace)
        values
        (1, 'warp', 'seanmonstar'),
        (1, 'actix', 'actix-web')
    ",
    )
        .execute(db)
        .await
        .expect("Failed to create entry");
    return result.rows_affected() == 2;
}

async fn create_project_stats(db: &PgPool) -> bool {
    let result = sqlx::query(
        "
        insert into project_stats (project_id, code_lines, unsafe_lines, created_at)
        values
        (1, 100, 10, '2020-01-01'),
        (1, 100, 11, '2021-01-01'),
        (2, 200, 20, '2020-01-01'),
        (2, 200, 11, '2021-01-01')
    ",
    )
        .execute(db)
        .await
        .expect("Failed to create entry");
    return result.rows_affected() == 4;
}


// /project/stats/update
#[tokio::test]
async fn test_project_stats_update() {
    let (address, db) = spawn_app().await;

    // Setup
    let _result = create_provider(&db).await;
    let _result = create_project(&db).await;

    // Positive assertion
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats/update", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    // This may need test timeout update.
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_project_stats_get_all() {
    let (address, db) = spawn_app().await;

    // Setup
    let _result = create_provider(&db).await;
    let _result = create_project(&db).await;
    let _result = create_project_stats(&db).await;

    let response = CLIENT
        .get(format!("{}/api/v1/project-stats", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let mut result: Value = response.json().await.unwrap();
    let project_stats: Vec<ProjectStats> =
        serde_json::from_value(result["project_stats"].take()).unwrap();
    assert_eq!(project_stats.len(), 2);
    assert_eq!(project_stats[0].unsafe_lines, 11);
    assert_eq!(project_stats[0].created_at, "2021-01-01");
    assert_eq!(project_stats[1].unsafe_lines, 11);
    assert_eq!(project_stats[1].created_at, "2021-01-01");
    // Todo: Add testing for the meta property.
}

#[tokio::test]
async fn test_providers_get_all() {
    let (address, db) = spawn_app().await;

    // Create an entry with sqlx since we dont have an endpoint.
    let _result = create_provider(&db).await;

    let response = CLIENT
        .get(format!("{}/api/v1/providers", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // // Assert
    assert!(response.status().is_success());
    let providers: Vec<Provider> = response.json().await.unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].url, "https://github.com");
}

#[tokio::test]
async fn test_providers_get_by_id() {
    let (address, db) = spawn_app().await;

    // Create an entry with sqlx since we dont have an endpoint.
    let _result = create_provider(&db).await;

    // Positive assertion
    let response = CLIENT
        .get(format!("{}/api/v1/providers/1", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let providers: Vec<Provider> = response.json().await.unwrap();
    assert_eq!(providers[0].url, "https://github.com");

    // Negative assertion
    let response = CLIENT
        .get(format!("{}/api/v1/providers/100", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let providers: Vec<Provider> = response.json().await.unwrap();
    assert_eq!(providers.len(), 0);
}

#[tokio::test]
async fn test_projects_get_all() {
    let (address, db) = spawn_app().await;

    // Create an entry with sqlx since we dont have an endpoint.
    // Create an entry with sqlx since we dont have an endpoint.
    let _result = create_provider(&db).await;
    let _result = create_project(&db).await;

    // Positive assertion
    let response = CLIENT
        .get(format!("{}/api/v1/projects", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let projects: Vec<Project> = response.json().await.unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[1].namespace, "actix-web");

    // Negative assertion
    let response = CLIENT
        .get(format!("{}/api/v1/projects/3", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let providers: Vec<Provider> = response.json().await.unwrap();
    assert_eq!(providers.len(), 0);
}

#[tokio::test]
async fn test_projects_get_by_id() {
    let (address, db) = spawn_app().await;

    // Create an entry with sqlx since we dont have an endpoint.
    let _result = create_provider(&db).await;
    let _result = create_project(&db).await;

    // Positive assertion
    let response = CLIENT
        .get(format!("{}/api/v1/projects/2", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let projects: Vec<Project> = response.json().await.unwrap();
    assert_eq!(projects[0].namespace, "actix-web");

    // Negative assertion
    let response = CLIENT
        .get(format!("{}/api/v1/projects/3", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let projects: Vec<Project> = response.json().await.unwrap();
    assert_eq!(projects.len(), 0);
}
