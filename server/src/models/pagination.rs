// Todo: Move this to another file.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PaginationOptions {
    pub(crate) limit: Option<u32>,
    pub(crate) page: Option<u32>,
}