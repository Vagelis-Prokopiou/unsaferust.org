use std::io::BufRead;
use std::sync::{Arc, Mutex};
use futures::future;
use tokio;
use crate::models::pagination::PaginationOptions;
use actix_web::web::{Data};
use actix_web::{web, HttpResponse, Responder};
use sqlx::{PgPool, Row};
use crate::models::project::{Project, ProjectStats, ProjectStatsDTO, ProjectWithUrl};
use crate::models::provider::Provider;
use anyhow::{Context};
use redis::{RedisResult};
use redis::AsyncCommands;

pub async fn health_check() -> impl Responder { HttpResponse::Ok() }

pub async fn index() -> impl Responder {
    return HttpResponse::build(actix_web::http::StatusCode::PERMANENT_REDIRECT)
        .append_header(("Location", "api/v1/projects"))
        .finish();
}

pub async fn project_stats_update(db: web::Data<PgPool>) -> Result<impl Responder, actix_web::Error> {
    let projects: Vec<ProjectWithUrl> = sqlx::query_as("
            select
            projects.*
            , providers.url
            from projects
            inner join providers on providers.id = projects.provider_id
         ")
        .fetch_all(db.as_ref())
        .await
        .with_context(|| "Failed to get projects from db".to_string())
        .map_err(actix_web::error::ErrorInternalServerError)?;

    // Prepare a variable to store the updated projects.
    let updated_projects: Arc<Mutex<Vec<ProjectStats>>> = Arc::new(Mutex::new(Vec::with_capacity(projects.len())));

    // Create the async tasks.
    let update_project_tasks: Vec<_> = projects
        .into_iter()
        .map(|project| {
            let updated_projects = updated_projects.clone();
            tokio::spawn(async move {
                let project_dir = &project.name;
                let project_url = format!("{}/{}/{}.git", project.url, project.namespace, project.name);
                let command = format!(
                    r#"
                    mkdir -p /tmp/rust_projects > /dev/null 2>&1;
                    cd /tmp/rust_projects;
                    if [ -d ./{project_dir} ]
                    then
                        cd ./{project_dir};
                        git pull > /dev/null 2>&1;
                        cd ..;
                    else
                        git clone {project_url};
                    fi
                    unsafe_lines=$(grep -r unsafe ./{project_dir} | grep '.*.rs' | grep -v '//' | grep -v 'forbid(unsafe_code)' | wc -l);

                    cd ./{project_dir};
                    code_lines=$(cloc . | grep Rust | awk '{{print $5}}');
                    echo "$unsafe_lines:$code_lines";
                    "#
                );
                let command_output = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .output()
                    .expect("Failed to execute std::process::Command");
                let command_output_string: String = String::from_utf8_lossy(&command_output.stdout).replace('\n', "");
                let data: Vec<i32> = command_output_string.split(':')
                    .map(|v| match v.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            println!("Failed to parse v {v} to i32 for {project_dir}");
                            0
                        }
                    })
                    .collect();
                assert_eq!(data.len(), 2, "The shell command did not resolve to 2 values (unsafe_lines:code_lines)");
                let unsafe_lines = data[0];
                let code_lines = data[1];

                updated_projects
                    .lock()
                    .unwrap()
                    .push(ProjectStats::new(
                        project.id,
                        code_lines,
                        unsafe_lines,
                        "".to_owned(),
                        "".to_owned())
                    );
            })
        })
        .collect();
    future::join_all(update_project_tasks).await;

    // Update the db.
    for updated_project in updated_projects.lock().unwrap().iter() {
        let project_id = updated_project.project_id;
        let code_lines = updated_project.code_lines;
        let unsafe_lines = updated_project.unsafe_lines;
        let query = format!("
            DO
            $do$
                BEGIN
                    if exists(
                                select project_id
                                from project_stats
                                where project_id = {project_id}
                                and unsafe_lines = {unsafe_lines}
                              )
                    then
                        update project_stats
                        set updated_at = current_date,
                        code_lines = {code_lines}
                        where project_id = {project_id}
                        and unsafe_lines = {unsafe_lines};
                    else
                        insert into project_stats (project_id, code_lines, unsafe_lines)
                        VALUES ({project_id}, {code_lines}, {unsafe_lines});
                    end if;
                END
            $do$");
        match sqlx::query(&query)
            .execute(db.as_ref())
            .await {
            Ok(_) => {}
            Err(_) => { println!("/project-stats/update failed for project_id {project_id}") }
        }
    }
    return Ok(HttpResponse::Ok());
}

pub async fn project_stats_get_by_id(
    db: Data<PgPool>,
    id: web::Path<i32>,
) -> Result<impl Responder, actix_web::Error> {
    // Here we return all the entries for this specific project.
    let project_stats: Vec<ProjectStats> = sqlx::query_as(
        "
        select
        project_id
        ,code_lines
        ,unsafe_lines
        ,COALESCE(cast(created_at as text), '') as created_at
        ,COALESCE(cast(updated_at as text), '') as updated_at
        from project_stats
        where project_id = $1
        order by created_at desc"
    )
        .bind(*id)
        .fetch_all(db.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    return Ok(web::Json(project_stats));
}

pub async fn project_stats_get_all(
    db: Data<PgPool>,
    redis: Data<std::sync::Mutex<redis::aio::Connection>>,
    pagination_options: web::Query<PaginationOptions>,
) -> Result<impl Responder, actix_web::Error> {
    let page = pagination_options.page.unwrap_or(1) - 1;
    let limit = pagination_options.limit.unwrap_or(50);
    let name = match &pagination_options.name {
        Some(v) => v.as_ref(),
        None => ""
    };
    let redis_key = format!("{page}_{limit}_{name}");
    let redis_value: RedisResult<String> = redis.lock().unwrap().get(&redis_key).await;
    if redis_value.is_ok() { return Ok(HttpResponse::Ok().body(redis_value.unwrap())); }

    let name_filtering = { if name.is_empty() { "" } else { "and name ilike concat('%', $1, '%')" } };
    let query = format!("
select t.project_id
     , t.name
     , t.url
     , t.code_lines
     , t.unsafe_lines
     , t.created_at
     , t.updated_at
     , COUNT(project_id) OVER () as total
from (
     select RANK() OVER (partition by ps.project_id ORDER BY ps.created_at desc) as rank_order
          , ps.project_id
          , p.name
          , concat(providers.url, '/', p.namespace, '/', p.name)                 as url
          , ps.code_lines
          , ps.unsafe_lines
          , COALESCE(cast(ps.created_at as text), '')                            as created_at
          , COALESCE(cast(ps.updated_at as text), '')                            as updated_at
     from project_stats as ps
     inner join projects as p on p.id = ps.project_id
     inner join providers on providers.id = p.provider_id
     order by p.name) as t
where t.rank_order = 1
{name_filtering}
limit {limit} offset ({limit} * {page});");
    let rows = sqlx::query(query.as_ref())
        .bind(name)
        .fetch_all(db.as_ref())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;

    let project_stats: Vec<ProjectStatsDTO> = rows
        .iter()
        .map(|row| {
            return ProjectStatsDTO::new(
                row.get("project_id"),
                row.get("name"),
                row.get("url"),
                row.get("code_lines"),
                row.get("unsafe_lines"),
                row.get("created_at"),
                row.get("updated_at"),
            );
        })
        .collect();

    // Todo: Add a type for this response.
    let total: i64 = if rows.is_empty() { 0 } else { rows[0].get("total") };
    let result = serde_json::json!({
        "project_stats": project_stats,
        "meta": { "total": total }
    });

    let json = serde_json::to_string(&result).unwrap();
    let _: String = redis.lock().unwrap().set(&redis_key, &json).await.unwrap();
    return Ok(HttpResponse::Ok().body(json));
}

pub async fn providers_get_all(db: web::Data<PgPool>) -> impl Responder {
    let providers: Vec<Provider> = sqlx::query_as("select * from providers")
        .fetch_all(db.as_ref())
        .await
        .expect("Failed to fetch providers");
    return web::Json(providers);
}

pub async fn providers_get_by_id(db: web::Data<PgPool>, id: web::Path<i32>) -> impl Responder {
    let providers: Vec<Provider> = sqlx::query_as("select * from providers where id = $1")
        .bind(*id)
        .fetch_all(db.as_ref())
        .await
        .expect("Failed to fetch providers");
    return web::Json(providers);
}

pub async fn projects_get_all(db: Data<PgPool>) -> impl Responder {
    let projects: Vec<Project> = sqlx::query_as("select * from projects")
        .fetch_all(db.as_ref())
        .await
        .expect("Failed to fetch projects");
    return web::Json(projects);
}

pub async fn projects_get_by_id(db: Data<PgPool>, id: web::Path<i32>) -> impl Responder {
    // Here we return all the entries for this specific project.
    let project_stats: Vec<Project> = sqlx::query_as("select * from projects where id = $1")
        .bind(*id)
        .fetch_all(db.as_ref())
        .await
        .expect("Failed to fetch projects");
    return web::Json(project_stats);
}

pub async fn projects_import(db: Data<PgPool>) -> Result<impl Responder, actix_web::Error> {
    let file = std::fs::File::open("./data/projects.txt")
        .map_err(actix_web::error::ErrorInternalServerError)?;
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
        // Todo: Log the failure.
        let _ = sqlx::query(&format!("
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
            .execute(db.as_ref())
            .await;
    }

    return Ok(HttpResponse::Ok());
}

pub async fn redis_flush(redis: Data<std::sync::Mutex<redis::aio::Connection>>) -> Result<impl Responder, actix_web::Error> {
    let mut guard = redis.lock();
    let connection = match guard.as_deref_mut() {
        Ok(v) => v,
        Err(_e) => return Err(actix_web::error::ErrorInternalServerError("/redis/purge: Failed to acquire connection to redis"))
    };
    // _result === OK
    let _result: String = redis::cmd("FLUSHDB")
        .query_async(connection)
        .await
        .map_err(actix_web::error::ErrorInternalServerError)?;
    return Ok(HttpResponse::Ok());
}