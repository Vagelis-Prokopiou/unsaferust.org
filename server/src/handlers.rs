use crate::{
    models::{
        pagination::Pagination,
        project::ProjectStatsWithMeta,
        project::{Project, ProjectStats, ProjectWithUrl},
        provider::Provider,
    },
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use futures::future;
use std::{io::BufRead, sync::Arc};
use crate::models::{CodeLines, UnsafeLines};

pub async fn healthCheck() -> StatusCode {
    return StatusCode::OK;
}

pub async fn updateProjectsStats(State(appState): State<AppState>) -> Result<(), StatusCode> {
    let projectsWithUrl: Vec<ProjectWithUrl> =
        appState.databaseService.getProjectsWithUrl().await?;

    // Prepare a variable to store the updated projects.
    let updatedProjects = Arc::new(tokio::sync::Mutex::new(Vec::with_capacity(
        projectsWithUrl.len(),
    )));

    // Create the async tasks.
    let update_project_tasks: Vec<_> = projectsWithUrl
        .into_iter()
        .map(|project| {
            let updated_projects = updatedProjects.clone();
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
                let unsafeLines = UnsafeLines::new(data[0]);
                let codeLines = CodeLines::new(data[1]);

                updated_projects
                    .lock()
                    .await
                    .push(ProjectStats::new(
                        project.id,
                        codeLines,
                        unsafeLines,
                        "".to_owned(),
                        "".to_owned())
                    );
            })
        })
        .collect();
    future::join_all(update_project_tasks).await;

    // Update the services.
    for updatedProject in updatedProjects.lock().await.iter() {
        let projectId = updatedProject.project_id;
        let codeLines = updatedProject.code_lines;
        let unsafeLines = updatedProject.unsafe_lines;
        appState
            .databaseService
            .updateProjectStatsById(
                projectId,
                codeLines,
                unsafeLines,
            )
            .await;
    }

    return Ok(());
}

pub async fn getProjectStatsById(
    State(appState): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<ProjectStats>>, StatusCode> {
    let result = appState.databaseService.getProjectsStatsById(id).await;
    if let Err(e) = result {
        let _ = appState
            .databaseService
            .logError(&format!("projectStatsGetById: {e}"))
            .await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    return Ok(Json(result.unwrap()));
}

pub async fn getProjectsStats(
    State(appState): State<AppState>,
    pagination: Query<Pagination>,
) -> Result<String, StatusCode> {
    let page = pagination.page.unwrap_or(1) - 1;
    let limit = pagination.limit.unwrap_or(50);
    let name = match &pagination.name {
        Some(v) => v.as_ref(),
        None => "",
    };
    let redisKey = format!("{page}_{limit}_{name}");
    let redisResult = appState.redisService.getKey(&redisKey).await;

    // Return the cached value if we have one.
    if let Err(e) = redisResult {
        let error = format!(
            "getProjectsStats(): RedisService::getKey() failed with error: {:?}",
            e
        );
        let _ = appState.databaseService.logError(&error).await;
    } else {
        return Ok(redisResult.unwrap());
    }

    let name_filtering = {
        if name.is_empty() {
            ""
        } else {
            "and name ilike concat('%', $1, '%')"
        }
    };
    let result: ProjectStatsWithMeta = appState
        .databaseService
        .getProjectsStats(name, name_filtering, limit, page)
        .await?;
    let json = serde_json::to_string(&result).unwrap();
    let redisResult = appState.redisService.setKey(&redisKey, &json).await;
    if let Err(e) = redisResult {
        let _ = appState
            .databaseService
            .logError(&format!(
                "Handlers::getProjectsStats() failed to save ro Redis: {:?}",
                e
            ))
            .await;
    };
    return Ok(json);
}

pub async fn getProviders(
    State(appState): State<AppState>,
) -> Result<Json<Vec<Provider>>, StatusCode> {
    let result = appState.databaseService.getProviders().await;
    if let Err(e) = result {
        let _ = appState
            .databaseService
            .logError(&format!("providers_get_all: {e}"))
            .await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    return Ok(Json(result.unwrap()));
}

pub async fn getProviderById(
    State(appState): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<Provider>>, StatusCode> {
    let result = appState.databaseService.getProviderById(id).await;
    if let Err(e) = result {
        let _ = appState
            .databaseService
            .logError(&format!("providers_get_by_id: {e}"))
            .await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    return Ok(Json(result.unwrap()));
}

pub async fn getProjects(
    State(appState): State<AppState>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let result = appState.databaseService.getProjects().await;
    if let Err(e) = result {
        let _ = appState
            .databaseService
            .logError(&format!("projects_get_all: {e}"))
            .await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    return Ok(Json(result.unwrap()));
}

pub async fn getProjectById(
    State(appState): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<Project>>, StatusCode> {
    let result = appState.databaseService.getProjectById(id).await;
    if let Err(e) = result {
        let _ = appState
            .databaseService
            .logError(&format!("projects_get_all: {e}"))
            .await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    return Ok(Json(result.unwrap()));
}

pub async fn projectsImport(State(appState): State<AppState>) -> Result<(), StatusCode> {
    let file = std::fs::File::open("./data/projects.txt")
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
    for line in std::io::BufReader::new(file).lines().flatten() {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('/').collect();
        if parts.len() != 5 {
            eprint!("Problem with line: {line}");
            continue;
        }

        let provider_url = format!("{}//{}", parts[0], parts[2]);
        let namespace = parts[3];
        let name = parts[4];
        appState
            .databaseService
            .createProject(&provider_url, namespace, name)
            .await?;
    }

    return Ok(());
}

pub async fn redisFlush(State(state): State<AppState>) -> Result<(), StatusCode> {
    let result = state.redisService.flush().await;
    if let Err(e) = result {
        let error = format!("redisFlush() failed with error {:?}", e);
        let _ = state.databaseService.logError(&error).await;
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    return Ok(());
}
