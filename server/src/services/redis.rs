#[derive(Clone)]
pub struct RedisService {
    //connection: PgPool,
}

impl RedisService {
    pub async fn new() -> RedisService { return RedisService {}; }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::time::{SystemTime, UNIX_EPOCH};
//
//     #[tokio::test]
//     async fn test_logErrorToFilesystem() {
//         let dbService = PostgresService::new(None).await;
//
//         let now = SystemTime::now();
//         let since_epoch = now.duration_since(UNIX_EPOCH).expect("Time went backwards");
//         let timestamp = since_epoch.as_secs();
//         let errorFile = format!("logs/{}.txt", timestamp);
//
//         let errorsArray = vec!["foo", "bar"];
//         for error in errorsArray {
//             let result = dbService.logErrorToFilesystem(error, Some(&errorFile));
//             assert!(result.is_ok());
//             let errors = std::fs::read_to_string(errorFile.clone()).unwrap();
//             assert!(errors.contains(error));
//         }
//     }
// }
