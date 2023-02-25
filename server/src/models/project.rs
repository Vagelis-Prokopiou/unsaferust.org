/// This is used to create new projects
/// and also provide the project info to the clients.
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct Project {
    pub(crate) id: i32,
    pub provider_id: i32,
    pub namespace: String,
    pub name: String,
}

/// This is used internally in stats/update.
/// The provider url is included, instead of the provider_id.
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct ProjectWithUrl {
    pub(crate) id: i32,
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) url: String,
}

/// This is used to provide all the project-stats related info to the clients.
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct ProjectStatsDTO {
    pub(crate) project_id: i32,
    pub name: String,
    pub(crate) url: String,
    pub(crate) code_lines: i32,
    pub unsafe_lines: i32,
    pub created_at: String,
    pub(crate) updated_at: String,
}

impl ProjectStatsDTO {
    pub fn new(
        project_id: i32,
        name: String,
        url: String,
        code_lines: i32,
        unsafe_lines: i32,
        created_at: String,
        updated_at: String,
    ) -> Self {
        return Self {
            project_id,
            name,
            url,
            code_lines,
            unsafe_lines,
            created_at,
            updated_at,
        };
    }
}

/// This is used internally. Todo; Add more info (where exactly).
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct ProjectStats {
    pub(crate) project_id: i32,
    pub(crate) code_lines: i32,
    pub unsafe_lines: i32,
    pub created_at: String,
    pub(crate) updated_at: String,
}

impl ProjectStats {
    pub fn new(
        project_id: i32,
        code_lines: i32,
        unsafe_lines: i32,
        created_at: String,
        updated_at: String,
    ) -> Self {
        return Self {
            project_id,
            code_lines,
            unsafe_lines,
            created_at,
            updated_at,
        };
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProjectStatsWithMeta {
    pub projectStats: Vec<ProjectStatsDTO>,
    pub meta: i64,
}
