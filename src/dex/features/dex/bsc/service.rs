use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    response::IntoResponse,
};
use futures::{SinkExt, StreamExt};
use repository::repositories::crypto::BlockchainClient;
use serde::Serialize;
use tokio::time::{interval, Duration};

use crate::shared::config::BlockchainConfig;

#[derive(Debug, Serialize, Clone)]
pub struct TokenDataMessage {
    pub price_usd: String,
    pub price_change_24h: f64,
    pub volume_24h: String,
    pub liquidity_usd: String,
    pub market_cap: String,
    pub timestamp: i64,
}

/// WebSocket handler for real-time BSC token data
/// Path: /dex/bsc/{token_address}
pub async fn handle_token_websocket(
    ws: WebSocketUpgrade,
    Path(token_address): Path<String>,
) -> impl IntoResponse {
    tracing::info!(
        "WebSocket connection request for BSC token: {}",
        token_address
    );
    ws.on_upgrade(move |socket| handle_socket(socket, token_address))
}

async fn handle_socket(socket: WebSocket, token_address: String) {
    let (mut sender, mut receiver) = socket.split();
    let chain_id = "bsc".to_string();

    // Load blockchain config
    let config = BlockchainConfig::new();
    let rpc_url = match config.get_rpc_url(&chain_id) {
        Some(url) => url,
        None => {
            tracing::error!("Unsupported chain: {}", chain_id);
            let _ = sender
                .send(Message::Text(
                    serde_json::json!({
                        "error": "Unsupported chain"
                    })
                    .to_string()
                    .into(),
                ))
                .await;
            return;
        }
    };

    // Create blockchain client
    let client = match BlockchainClient::new(rpc_url).await {
        Ok(client) => client,
        Err(e) => {
            tracing::error!("Failed to create blockchain client: {}", e);
            let _ = sender
                .send(Message::Text(
                    serde_json::json!({
                        "error": "Failed to connect to blockchain"
                    })
                    .to_string()
                    .into(),
                ))
                .await;
            return;
        }
    };

    // Create interval for periodic updates (every 3 seconds)
    let mut update_interval = interval(Duration::from_secs(3));

    // Clone token_address for the spawned task
    let token_address_clone = token_address.clone();

    // Main loop handling both updates and incoming messages
    loop {
        tokio::select! {
            _ = update_interval.tick() => {
                // Fetch token data
                let token_data = match fetch_token_data(&client, &token_address_clone, &config).await {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::error!("Failed to fetch token data: {}", e);
                        continue;
                    }
                };

                // Send update to client
                let message = match serde_json::to_string(&token_data) {
                    Ok(json) => Message::Text(json.into()),
                    Err(e) => {
                        tracing::error!("Failed to serialize token data: {}", e);
                        continue;
                    }
                };

                if sender.send(message).await.is_err() {
                    tracing::info!("Client disconnected");
                    break;
                }
            }

            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Close(_))) => {
                        tracing::info!("Client closed connection");
                        break;
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if sender.send(Message::Pong(data)).await.is_err() {
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        break;
                    }
                    None => {
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    tracing::info!("WebSocket connection closed for token: {}", token_address);
}

async fn fetch_token_data(
    client: &BlockchainClient,
    token_address: &str,
    config: &BlockchainConfig,
) -> Result<TokenDataMessage, Box<dyn std::error::Error + Send + Sync>> {
    // Fetch token metadata
    let metadata = client.get_token_metadata(token_address).await?;

    // Calculate token price from DEX pairs
    let price_data = client
        .calculate_token_price(
            token_address,
            &config.dex_contracts.pancakeswap_v2_factory,
            config.get_wbnb_address(),
            config.get_busd_address(),
        )
        .await?;

    // Calculate market cap (price * total supply)
    let total_supply_f64 =
        metadata.total_supply.as_u128() as f64 / 10f64.powi(metadata.decimals as i32);
    let market_cap = price_data.price_usd * total_supply_f64;

    Ok(TokenDataMessage {
        price_usd: price_data.price_usd.to_string(),
        price_change_24h: 0.0,
        volume_24h: "0".to_string(),
        liquidity_usd: price_data.liquidity_usd.to_string(),
        market_cap: market_cap.to_string(),
        timestamp: chrono::Utc::now().timestamp(),
    })
}
