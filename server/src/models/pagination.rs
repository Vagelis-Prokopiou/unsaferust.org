// Todo: Move this to another file.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Pagination {
    pub(crate) limit: Option<u32>,
    pub(crate) page: Option<u32>,
    pub(crate) name: Option<String>,
}
