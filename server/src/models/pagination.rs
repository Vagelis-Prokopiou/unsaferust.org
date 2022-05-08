// Todo: Move this to another file.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PaginationOptions {
    #[serde(rename(deserialize = "id"))]
    pub(crate) project_id: Option<u32>,
    pub(crate) limit: Option<u32>,
    pub(crate) direction: Option<String>,
}

// use serde::{Deserialize, Serialize};
//
// #[derive(Serialize, Deserialize, Debug)]
// pub struct User {
//     pub first_name: String,
//     pub last_name: String,
//     pub mail: String,
// }
//
// impl User {
//     pub fn new(first_name: String, last_name: String, mail: String) -> Self {
//         Self {
//             first_name,
//             last_name,
//             mail,
//         }
//     }
// }
