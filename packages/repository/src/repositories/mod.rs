pub mod crypto;
pub mod encryption;
pub mod queue;

use std::sync::Arc;

// Centralized dependency hub for providers and data repositories
#[derive(Clone)]
pub struct Repositories {
    // Shared services
    pub encryption: Arc<encryption::EncryptionRepository>,
    pub queue: Arc<queue::rabbitmq::RabbitMQRepository>,
    pub crypto: Arc<crypto::CryptoRepository>,
}

impl Repositories {
    pub fn new() -> Self {
        // Encryption uses a sane default; override as needed in callers
        let encryption: Arc<encryption::EncryptionRepository> =
            Arc::new(encryption::EncryptionRepository::default());

        // Queue and cache endpoints from env with defaults
        let rabbitmq_url =
            std::env::var("AMQP_URL").unwrap_or_else(|_| "amqp://127.0.0.1:5672/%2f".to_string());

        let queue: Arc<queue::rabbitmq::RabbitMQRepository> =
            Arc::new(queue::rabbitmq::RabbitMQRepository::new(rabbitmq_url));

        let crypto: Arc<crypto::CryptoRepository> = Arc::new(crypto::CryptoRepository::default());

        Self {
            encryption,
            queue,
            crypto,
        }
    }
}
