use async_trait::async_trait;
use lapin::{options::{BasicAckOptions, BasicConsumeOptions, BasicNackOptions, BasicPublishOptions, QueueDeclareOptions}, types::FieldTable, BasicProperties, Channel, Connection, ConnectionProperties};
use crate::shared::data::repositories::queue::{QueueRepositoryTrait};
use crate::shared::data::repositories::queue::data::QueueError;

pub struct RabbitMQRepository {
    connection_url: String,
}

impl RabbitMQRepository {
    pub fn new(connection_url: String) -> Self {
        Self { connection_url }
    }

    async fn get_channel(&self) -> Result<Channel, QueueError> {
        let conn = Connection::connect(&self.connection_url, ConnectionProperties::default())
            .await
            .map_err(|e| QueueError::ConnectionError(format!("RabbitMQ connect error: {}", e)))?;
        conn.create_channel()
            .await
            .map_err(|e| QueueError::ConnectionError(format!("Create channel error: {}", e)))
    }
}

#[async_trait]
impl QueueRepositoryTrait for RabbitMQRepository {
    async fn consume<F>(&self, queue: &str, handler: F) -> Result<(), QueueError>
    where
        F: Fn(Vec<u8>) -> Result<(), QueueError> + Send + Sync,
    {
        let channel = self.get_channel().await?;
        channel
            .queue_declare(
                queue,
                QueueDeclareOptions { durable: true, ..Default::default() },
                FieldTable::default(),
            )
            .await
            .map_err(|e| QueueError::ConsumeError(format!("Queue declare error: {}", e)))?;

        let mut consumer = channel
            .basic_consume(
                queue,
                "worker-consumer",
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await
            .map_err(|e| QueueError::ConsumeError(format!("Consume error: {}", e)))?;

        while let Some(delivery) = consumer.next().await {
            let delivery = delivery
                .map_err(|e| QueueError::ConsumeError(format!("Delivery error: {}", e)))?;
            let tag = delivery.delivery_tag;
            match handler(delivery.data.clone()) {
                Ok(_) => {
                    channel
                        .basic_ack(tag, BasicAckOptions::default())
                        .await
                        .map_err(|e| QueueError::AcknowledgeError(format!("Ack error: {}", e)))?;
                }
                Err(err) => {
                    channel
                        .basic_nack(tag, BasicNackOptions { requeue: true, ..Default::default() })
                        .await
                        .map_err(|e| QueueError::QueueError(format!("Nack error: {}. original: {}", e, err)))?;
                }
            }
        }

        Ok(())
    }

    async fn acknowledge(&self, delivery_tag: u64) -> Result<(), QueueError> {
        let channel = self.get_channel().await?;
        channel
            .basic_ack(delivery_tag, BasicAckOptions::default())
            .await
            .map_err(|e| QueueError::AcknowledgeError(format!("Ack error: {}", e)))
    }

    async fn reject(&self, delivery_tag: u64, requeue: bool) -> Result<(), QueueError> {
        let channel = self.get_channel().await?;
        channel
            .basic_nack(delivery_tag, BasicNackOptions { requeue, ..Default::default() })
            .await
            .map_err(|e| QueueError::QueueError(format!("Nack error: {}", e)))
    }

    async fn publish(&self, queue: &str, message: &[u8]) -> Result<(), QueueError> {
        let channel = self.get_channel().await?;
        channel
            .queue_declare(
                queue,
                QueueDeclareOptions { durable: true, ..Default::default() },
                FieldTable::default(),
            )
            .await
            .map_err(|e| QueueError::PublishError(format!("Queue declare error: {}", e)))?;

        channel
            .basic_publish(
                "",
                queue,
                BasicPublishOptions::default(),
                message,
                BasicProperties::default(),
            )
            .await
            .map_err(|e| QueueError::PublishError(format!("Publish error: {}", e)))?;
        Ok(())
    }
}

use futures::StreamExt;

