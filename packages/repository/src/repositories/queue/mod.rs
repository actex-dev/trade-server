use async_trait::async_trait;
use data::QueueError;

pub mod data;
pub mod rabbitmq;

#[allow(dead_code)]
#[async_trait]
pub trait QueueRepositoryTrait: Send + Sync {
    /// Consume messages from a queue with a handler function
    async fn consume<F>(&self, queue: &str, handler: F) -> Result<(), QueueError>
    where
        F: Fn(Vec<u8>) -> Result<(), QueueError> + Send + Sync;

    /// Acknowledge a message has been processed
    async fn acknowledge(&self, delivery_tag: u64) -> Result<(), QueueError>;

    /// Reject a message (nack)
    async fn reject(&self, delivery_tag: u64, requeue: bool) -> Result<(), QueueError>;

    /// Publish a message to a queue (optional, for replies/acks)
    async fn publish(&self, queue: &str, message: &[u8]) -> Result<(), QueueError>;
}

