use crate::models::{configuration::DatabaseSettings, project::*, provider::Provider};
use axum::http::StatusCode;
use sqlx::Row;
use sqlx::{postgres::PgPoolOptions, Error, PgPool};
use std::{fs::OpenOptions, io::Write};
use crate::utils::getDate;


#[derive(Clone)]
pub struct PostgresService {
    pub connection: PgPool,
}

impl PostgresService {
    pub async fn new(con: Option<PgPool>) -> Self {
        if con.is_some() {
            return Self {
                connection: con.unwrap(),
            };
        }

        // Prepare the variables that the run method needs.
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
        let dbSettings = DatabaseSettings::new(
            db_user,
            db_password,
            db_port,
            db_host,
            db_name,
            db_max_connections,
        );

        let connection = PgPoolOptions::new()
            .max_connections(dbSettings.max_connections)
            .connect(&dbSettings.get_connection_string_with_db())
            .await
            .expect("PgPoolOptions initialization failed");

        return Self { connection };
    }

    pub fn logErrorToFilesystem(&self, error: &str, errorFile: Option<&str>) -> Result<(), &str> {
        // Todo: Do this once on startup.
        let dirName = "logs";
        let _ = std::fs::create_dir(dirName);

        let errorFile = match errorFile {
            None => format!("{dirName}/error_log.txt"),
            Some(v) => {
                if !v.contains(dirName) {
                    format!("{dirName}/{}", v)
                } else {
                    v.to_string()
                }
            }
        };

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(errorFile)
            .map_err(|_| "")?;

        let date = getDate();
        file.write_all(format!("{date}: {error}\n").as_bytes()).map_err(|_| "")?;
        return Ok(());
    }

    pub async fn logError(&self, error: &str) -> Result<(), String> {
        let dbResult = sqlx::query("insert into error_log(error) Values($1)")
            .bind(error)
            .execute(&self.connection)
            .await;

        if let Err(e) = dbResult {
            let err = format!(
                "logError failed to write to services with error {:?} \n for initial error: {}",
                e, error
            );
            let _ = self.logErrorToFilesystem(&err, None);
        }
        return Ok(());
    }

    pub async fn getProjectsWithUrl(&self) -> Result<Vec<ProjectWithUrl>, StatusCode> {
        let result: Result<Vec<ProjectWithUrl>, Error> = sqlx::query_as(
            "
            select
            projects.*
            , providers.url
            from projects
            inner join providers on providers.id = projects.provider_id
         ",
        )
            .fetch_all(&self.connection)
            .await;
        if let Err(e) = result {
            let error = format!(
                "DatabaseService::getProjectsWithUrl failed to retrieve data, with error: {:?}",
                e
            );
            let _ = self.logError(&error).await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        return Ok(result.unwrap());
    }

    pub async fn getProjectsStatsById(&self, id: i32) -> Result<Vec<ProjectStats>, String> {
        let projectStats: Vec<ProjectStats> = sqlx::query_as(
            "
        select
        project_id
        ,code_lines
        ,unsafe_lines
        ,COALESCE(cast(created_at as text), '') as created_at
        ,COALESCE(cast(updated_at as text), '') as updated_at
        from project_stats
        where project_id = $1
        order by created_at desc",
        )
            .bind(id)
            .fetch_all(&self.connection)
            .await
            .map_err(|e| format!("DatabaseService.getProjectsStats failed: {:?}", e))?;
        return Ok(projectStats);
    }

    pub async fn getProjectsStats(
        &self,
        name: &str,
        name_filtering: &str,
        limit: u32,
        page: u32,
    ) -> Result<ProjectStatsWithMeta, StatusCode> {
        let query = format!(
            "
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
limit {limit} offset ({limit} * {page});"
        );
        let rowsResult = sqlx::query(query.as_ref())
            .bind(name)
            .fetch_all(&self.connection)
            .await;

        if let Err(e) = rowsResult {
            let _ = self
                .logError(&format!(
                    "DatabaseService::getProjectsStats() failed to get rows: {:?}",
                    e
                ))
                .await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };

        let rows = rowsResult.unwrap();
        let projectStats: Vec<ProjectStatsDTO> = rows
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
        let total: i64 = if rows.is_empty() {
            0
        } else {
            rows[0].get("total")
        };

        let result = ProjectStatsWithMeta {
            projectStats,
            meta: total,
        };
        return Ok(result);
    }

    pub async fn updateProjectStatsById(
        &self,
        project_id: i32,
        code_lines: i32,
        unsafe_lines: i32,
    ) {
        let query = format!(
            "
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
            $do$"
        );

        let result = sqlx::query(&query).execute(&self.connection).await;
        if let Err(e) = result {
            let _ = self
                .logError(&format!("Failed to update projectStats with e: {:?}", e))
                .await;
        }
    }

    pub async fn getProviders(&self) -> Result<Vec<Provider>, String> {
        let providers: Vec<Provider> = sqlx::query_as("select * from providers")
            .fetch_all(&self.connection)
            .await
            .map_err(|e| format!("DatabaseService.get_providers failed: {:?}", e))?;
        return Ok(providers);
    }

    pub async fn getProviderById(&self, id: i32) -> Result<Vec<Provider>, String> {
        let providers: Vec<Provider> = sqlx::query_as("select * from providers where id = $1")
            .bind(id)
            .fetch_all(&self.connection)
            .await
            .map_err(|e| format!("DatabaseService.get_provider_by_id failed: {:?}", e))?;
        return Ok(providers);
    }

    pub async fn getProjects(&self) -> Result<Vec<Project>, String> {
        let projects: Vec<Project> = sqlx::query_as("select * from projects")
            .fetch_all(&self.connection)
            .await
            .map_err(|e| format!("DatabaseService.get_projects failed: {:?}", e))?;
        return Ok(projects);
    }

    pub async fn getProjectById(&self, id: i32) -> Result<Vec<Project>, String> {
        let projects: Vec<Project> = sqlx::query_as("select * from projects where id = $1")
            .bind(id)
            .fetch_all(&self.connection)
            .await
            .map_err(|e| format!("DatabaseService.get_project_by_id failed: {:?}", e))?;
        return Ok(projects);
    }

    pub async fn createProject(
        &self,
        providerUrl: &str,
        namespace: &str,
        name: &str,
    ) -> Result<(), StatusCode> {
        let result = sqlx::query(&format!(
            "
                    do
                    $do$
                    begin
                    if not exists (select id from projects where name = '{name}') then
                        insert into projects (provider_id, namespace, name)
                         VALUES (
                         (select id from providers where url = '{providerUrl}'),
                         '{namespace}',
                         '{name}'
                         );
                    end if;
                    end
                    $do$
        "
        ))
            .execute(&self.connection)
            .await;

        if let Err(e) = result {
            let _ = self
                .logError(&format!("DatabaseService::createProject failed: {:?}", e))
                .await;
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;

    #[tokio::test]
    async fn testLogErrorToFilesystem() {
        let databaseService = PostgresService::new(None).await;
        let timestamp = utils::getTimestamp();
        let errorFile = format!("logs/{}.txt", timestamp);

        let errorsArray = vec!["foo", "bar"];
        for error in errorsArray {
            let result = databaseService.logErrorToFilesystem(error, Some(&errorFile));
            assert!(result.is_ok());
            let errors = std::fs::read_to_string(errorFile.clone()).unwrap();
            assert!(errors.contains(error));
        }
    }
}
