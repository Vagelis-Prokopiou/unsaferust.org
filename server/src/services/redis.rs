use redis::{AsyncCommands, RedisError};
use std::ops::DerefMut;

#[derive(Clone)]
pub struct RedisService {
    pub connection: std::sync::Arc<tokio::sync::Mutex<redis::aio::Connection>>,
}

impl RedisService {
    pub async fn new() -> RedisService {
        let redisHost = std::env::var("REDIS_HOST").expect("env::var REDIS_HOST failed");
        let redisClient = redis::Client::open(format!("redis://{redisHost}"))
            .expect("Failed to create Redis client");
        let connection: redis::aio::Connection = redisClient
            .get_async_connection()
            .await
            .expect("RedisService::new() failed");
        let connection = std::sync::Arc::new(tokio::sync::Mutex::new(connection));
        return RedisService { connection };
    }

    // Todo: Add delete
    // Todo: Add tests

    pub async fn setKey(&self, key: &str, value: &str) -> Result<String, RedisError> {
        let mut guard = self.connection.lock().await;
        let connection = guard.deref_mut();
        let value: String = connection.set(key, value).await?;
        return Ok(value);
    }

    pub async fn getKey(&self, key: &str) -> Result<String, RedisError> {
        let mut guard = self.connection.lock().await;
        let connection = guard.deref_mut();
        let value: String = connection.get(key).await?;
        return Ok(value);
    }

    pub async fn flush(&self) -> Result<String, RedisError> {
        let mut guard = self.connection.lock().await;
        let connection = guard.deref_mut();
        let value = redis::cmd("FLUSHDB").query_async(connection).await?;
        return Ok(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils;

    #[tokio::test]
    async fn testRedisSetAndGet() {
        let redisService = RedisService::new().await;
        let timestamp = utils::getTimestamp();
        let key = format!("{}_key", timestamp);
        let value = format!("{}_value", timestamp);

        // Setter
        let result = redisService.setKey(&key, &value).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "OK");

        // Getter
        let result = redisService.getKey(&key).await;
        assert!(result.is_ok());
        let retrievedValue = result.unwrap();
        assert_eq!(value, retrievedValue);
    }

    #[tokio::test]
    async fn testRedisFlush() {
        // This test creates global mutation and makes the previous test fail.
        // Run it only locally if you need to.

        // let redisService = RedisService::new().await;
        // let timestamp = utils::getTimestamp();
        // let key = format!("{}_key", timestamp);
        // let value = format!("{}_value", timestamp);
        //
        // // Setter
        // let result = redisService.setKey(&key, &value).await;
        // assert!(result.is_ok());
        // assert_eq!(result.unwrap(), "OK");
        //
        // // Flush
        // let result = redisService.flush().await;
        // assert!(result.is_ok());
        // assert_eq!(result.unwrap(), "OK");
        //
        // // Getter
        // let result = redisService.getKey(&key).await;
        // assert!(result.is_err());
        // assert!(format!("{:?}", result).contains("nil"));
    }
}
