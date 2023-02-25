pub mod configuration;
pub mod pagination;
pub mod project;
pub mod provider;
pub mod appState;

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct CodeLines(pub i32);

impl CodeLines { pub fn new(lines: i32) -> Self { return Self(lines); } }

impl From<i32> for CodeLines { fn from(value: i32) -> Self { return Self(value); } }

#[derive(Copy, Clone, Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct UnsafeLines(pub i32);

impl UnsafeLines { pub fn new(lines: i32) -> Self { return Self(lines); } }

impl From<i32> for UnsafeLines { fn from(value: i32) -> Self { return Self(value); } }
