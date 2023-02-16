#[derive(Debug, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct Provider {
    pub(crate) id: i32,
    pub url: String,
}

impl Provider {
    pub fn new(id: i32, url: &str) -> Self {
        return Self {
            id,
            url: url.to_owned(),
        };
    }
}
