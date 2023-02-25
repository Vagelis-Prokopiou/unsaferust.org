use crate::services::{postgres::PostgresService, redis::RedisService};

#[derive(Clone)]
pub struct AppState {
    pub(crate) redisService: RedisService,
    pub(crate) databaseService: PostgresService,
}