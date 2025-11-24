use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum QueueError {
    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Consume error: {0}")]
    ConsumeError(String),

    #[error("Publish error: {0}")]
    PublishError(String),

    #[error("Acknowledge error: {0}")]
    AcknowledgeError(String),

    #[error("Message deserialization error: {0}")]
    DeserializationError(String),

    #[error("Queue error: {0}")]
    QueueError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

#[allow(dead_code)]
pub type MessageHandler = Box<dyn Fn(Vec<u8>) -> Result<(), QueueError> + Send + Sync>;

