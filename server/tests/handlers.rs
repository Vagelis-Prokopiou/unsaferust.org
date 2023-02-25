#![allow(non_snake_case)]

use hyper::StatusCode;
use serde_json::Value;
use sqlx::Executor;
use sqlx::{Connection, PgConnection, PgPool};
use std::net::TcpListener;
use unsaferust::models::configuration::DatabaseSettings;
use unsaferust::models::project::{Project, ProjectStats, ProjectStatsWithMeta};
use unsaferust::models::provider::Provider;
use unsaferust::services::postgres::PostgresService;
use unsaferust::services::redis::RedisService;
use uuid::Uuid;

lazy_static::lazy_static! { static ref CLIENT: reqwest::Client = reqwest::Client::new(); }

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

    let databaseService = PostgresService::new(Some(connection_pool.clone())).await;
    let redisService = RedisService::new().await;
    let server =
        unsaferust::run(listener, redisService, databaseService).expect("Failed to bind address");
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
    let result = sqlx::query("insert into providers (url) values ('https://github.com')")
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

/*****************/
/* project-stats */
/*****************/
// #[tokio::test]
// async fn test_project_stats_update() {
//     let (address, services) = spawn_app().await;
//
//     // Setup
//     let _result = create_provider(&services).await;
//     let _result = create_project(&services).await;
//
//     // Positive assertion
//     let response = CLIENT
//         .get(format!("{}/api/v1/project-stats/update", &address))
//         .send()
//         .await
//         .expect("Failed to execute request.");
//
//     // Assert
//     // This may need test timeout update.
//     assert!(response.status().is_success());
// }

/************/
/* projects */
/************/
#[tokio::test]
async fn test_projects_get_all() {
    let (address, db) = spawn_app().await;

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
    let projects: Vec<Project> = response.json().await.unwrap();
    assert_eq!(projects.len(), 0);
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

#[tokio::test]
async fn test_health_check() {
    let (address, _db) = spawn_app().await;
    let response = CLIENT
        .get(format!("{}/api/health_check", &address))
        .send()
        .await
        .expect("Failed to execute request.");
    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_project_stats_pagination() {
    let (address, db) = spawn_app().await;

    // Setup
    let _result = create_provider(&db).await;
    for i in 1..10 {
        let _ = sqlx::query(&format!(
            "
            insert into projects (provider_id, name, namespace)
            values (1, 'name_{i}', 'namespace_{i}')"
        ))
        .execute(&db)
        .await
        .expect("Failed to create entry");

        let _ = sqlx::query(&format!(
            "
            insert into project_stats (project_id, code_lines, unsafe_lines)
            values ({i}, '{i}', '{i}')"
        ))
        .execute(&db)
        .await
        .expect("Failed to create entry");
    }

    // Start requesting
    // 1. Get all items.
    redis_flush(&address).await;
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats?page=1&limit=25", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 9);

    // 1. Get 3 items per page (page 1).
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats?page=1&limit=3", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 3);
    assert_eq!(response.projectStats[0].name, "name_1");
    assert_eq!(response.projectStats[1].name, "name_2");
    assert_eq!(response.projectStats[2].name, "name_3");

    // 1. Get 3 items per page (page 3).
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats?page=3&limit=3", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 3);
    assert_eq!(response.projectStats[0].name, "name_7");
    assert_eq!(response.projectStats[1].name, "name_8");
    assert_eq!(response.projectStats[2].name, "name_9");

    // 1. Get 4 items per page (page 2).
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats?page=2&limit=4", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 4);
    assert_eq!(response.projectStats[0].name, "name_5");
    assert_eq!(response.projectStats[1].name, "name_6");
    assert_eq!(response.projectStats[2].name, "name_7");
    assert_eq!(response.projectStats[3].name, "name_8");

    // 1. Get 8 items per page (page 2).
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats?page=2&limit=8", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 1);
    assert_eq!(response.projectStats[0].name, "name_9");
}

#[tokio::test]
async fn testProjectStatsPaginationWithName() {
    let (address, db) = spawn_app().await;

    // Setup
    let _result = create_provider(&db).await;
    // Insert foo
    for i in 1..10 {
        let _ = sqlx::query(&format!(
            "
            insert into projects (id, provider_id, name, namespace)
            values ({i}, 1, 'foo_{i}', 'namespace_{i}')"
        ))
        .execute(&db)
        .await
        .expect("Failed to create entry");

        let _ = sqlx::query(&format!(
            "
            insert into project_stats (project_id, code_lines, unsafe_lines)
            values ({i}, '{i}', '{i}')
            "
        ))
        .execute(&db)
        .await
        .expect("Failed to create entry");
    }

    // Insert bar
    for i in 1..10 {
        let _ = sqlx::query(&format!(
            "
            insert into projects (id, provider_id, name, namespace)
            values (({i} + 9), 1, 'bar_{i}', 'namespace_{i}')"
        ))
        .execute(&db)
        .await
        .expect("Failed to create entry");

        let _ = sqlx::query(&format!(
            "
            insert into project_stats (project_id, code_lines, unsafe_lines)
            values (({i} + 9), '{i}', '{i}')
            "
        ))
        .execute(&db)
        .await
        .expect("Failed to create entry");
    }

    // Start requesting
    // 1. Get all items.
    redis_flush(&address).await;
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats?page=1&limit=25", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 18);

    // 1. Get 3 foo items per page (page 1).
    let response = CLIENT
        .get(format!(
            "{}/api/v1/project-stats?page=1&limit=3&name=foo",
            &address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 3);
    assert_eq!(response.projectStats[0].name, "foo_1");
    assert_eq!(response.projectStats[1].name, "foo_2");
    assert_eq!(response.projectStats[2].name, "foo_3");

    // 1. Get 3 foo items per page (page 3).
    let response = CLIENT
        .get(format!(
            "{}/api/v1/project-stats?page=3&limit=3&name=foo",
            &address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 3);
    assert_eq!(response.projectStats[0].name, "foo_7");
    assert_eq!(response.projectStats[1].name, "foo_8");
    assert_eq!(response.projectStats[2].name, "foo_9");

    // 1. Get 4 bar items per page (page 2).
    let response = CLIENT
        .get(format!(
            "{}/api/v1/project-stats?page=2&limit=4&name=bar",
            &address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 4);
    assert_eq!(response.projectStats[0].name, "bar_5");
    assert_eq!(response.projectStats[1].name, "bar_6");
    assert_eq!(response.projectStats[2].name, "bar_7");
    assert_eq!(response.projectStats[3].name, "bar_8");

    // 1. Get 8 bar items per page (page 2).
    let response = CLIENT
        .get(format!(
            "{}/api/v1/project-stats?page=2&limit=8&name=bar",
            &address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 1);
    assert_eq!(response.projectStats[0].name, "bar_9");

    // Get a non existent item
    let response = CLIENT
        .get(format!(
            "{}/api/v1/project-stats?page=2&limit=8&name=hello",
            &address
        ))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
    assert!(response.status().is_success());
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 0);
}

async fn redis_flush(address: &String) {
    let _purge_result = CLIENT
        .get(format!("{}/api/redis/flush", &address))
        .send()
        .await
        .expect("Failed to execute request.");
    assert_eq!(_purge_result.status(), 200);
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
    let result: Value = response.json().await.unwrap();
    let response: ProjectStatsWithMeta = serde_json::from_value(result).unwrap();
    assert_eq!(response.projectStats.len(), 2);
    assert_eq!(response.projectStats[0].unsafe_lines, 11);
    assert_eq!(response.projectStats[0].created_at, "2021-01-01");
    assert_eq!(response.projectStats[1].unsafe_lines, 11);
    assert_eq!(response.projectStats[1].created_at, "2021-01-01");
    assert_eq!(response.meta, 2);

    // Todo: Add testing for the meta property.
}

#[tokio::test]
async fn test_project_stats_get_by_id() {
    let (address, db) = spawn_app().await;

    // Setup
    let _result = create_provider(&db).await;
    let _result = create_project(&db).await;
    let _result = create_project_stats(&db).await;

    // Testing with project id 1.
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats/1", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert (we must get 2 records for id 1 in created_at desc)
    assert!(response.status().is_success());
    let project_stats: Vec<ProjectStats> = response.json().await.unwrap();
    assert_eq!(project_stats.len(), 2);
    assert_eq!(project_stats[0].unsafe_lines, 11);
    assert_eq!(project_stats[0].created_at, "2021-01-01");
    assert_eq!(project_stats[1].unsafe_lines, 10);
    assert_eq!(project_stats[1].created_at, "2020-01-01");

    // Testing with project id 2.
    let response = CLIENT
        .get(format!("{}/api/v1/project-stats/2", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert (we must get 2 records for id 2 in created_at desc)
    assert!(response.status().is_success());
    let project_stats: Vec<ProjectStats> = response.json().await.unwrap();
    assert_eq!(project_stats.len(), 2);
    assert_eq!(project_stats[0].unsafe_lines, 11);
    assert_eq!(project_stats[0].created_at, "2021-01-01");
    assert_eq!(project_stats[1].unsafe_lines, 20);
    assert_eq!(project_stats[1].created_at, "2020-01-01");
}

#[tokio::test]
async fn test_non_existing_routes() {
    let (address, _db) = spawn_app().await;

    let arbitrary_paths = vec!["", "foo", "foo/bar", "foo/bar/baz"];
    for arbitrary_path in arbitrary_paths {
        let response = CLIENT
            .get(format!("{}/{}", &address, arbitrary_path))
            .send()
            .await
            .expect("Failed to execute request.");

        // Assert
        let status = response.status();
        assert_eq!(status, StatusCode::NOT_FOUND);
    }
}

/*************/
/* providers */
/*************/
#[tokio::test]
async fn test_providers_get_all() {
    let (address, db) = spawn_app().await;

    // Create an entry with sqlx since we dont have an endpoint.
    let _ = create_provider(&db).await;

    let response = CLIENT
        .get(format!("{}/api/v1/providers", &address))
        .send()
        .await
        .expect("Failed to execute request.");

    // Assert
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
