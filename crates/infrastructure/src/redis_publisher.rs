use application::{error::ApplicationError, ports::EventPublisher};
use domain::TaskEvent;
use redis::{AsyncCommands, aio::ConnectionManager};
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct RedisPublisher {
    connection: Mutex<ConnectionManager>,
    channel: String,
}
impl RedisPublisher {
    pub async fn open(redis_url: &str, channel: impl Into<String>) -> redis::RedisResult<Self> {
        let client = redis::Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;

        Ok(Self {
            connection: Mutex::new(connection),
            channel: channel.into(),
        })
    }
}

#[async_trait::async_trait]
impl EventPublisher for RedisPublisher {
    async fn publish(&self, event: TaskEvent) -> Result<(), ApplicationError> {
        let mut connection = self.connection.lock().await;

        let serialized = serde_json::to_string(&event)
            .map_err(|e| ApplicationError::Repository(e.to_string()))?;
        let result: redis::RedisResult<usize> = connection.publish(&self.channel, serialized).await;

        match result {
            Ok(n) => {
                info!(channel = %self.channel, count = n, "event published.");
                Ok(())
            }
            Err(e) => {
                error!(channel = %self.channel, error = %e, "failed to publish event.");
                Err(ApplicationError::Repository(e.to_string()))
            }
        }
    }
}
