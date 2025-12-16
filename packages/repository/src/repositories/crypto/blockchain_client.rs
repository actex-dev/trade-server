use ethers::{
    prelude::*,
    providers::{Http, Provider},
    types::{Address, U256},
};
use std::sync::Arc;

// ERC20 Token ABI (minimal)
abigen!(
    ERC20,
    r#"[
        function name() external view returns (string)
        function symbol() external view returns (string)
        function decimals() external view returns (uint8)
        function totalSupply() external view returns (uint256)
        function balanceOf(address account) external view returns (uint256)
    ]"#
);

// Uniswap V2 Pair ABI (for PancakeSwap)
abigen!(
    UniswapV2Pair,
    r#"[
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast)
        function token0() external view returns (address)
        function token1() external view returns (address)
    ]"#
);

// Uniswap V2 Factory ABI
abigen!(
    UniswapV2Factory,
    r#"[
        function getPair(address tokenA, address tokenB) external view returns (address pair)
    ]"#
);

pub struct BlockchainClient {
    provider: Arc<Provider<Http>>,
}

impl BlockchainClient {
    pub async fn new(rpc_url: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let provider = Provider::<Http>::try_from(rpc_url)?;
        Ok(Self {
            provider: Arc::new(provider)
        })
    }

    /// Fetch token metadata (name, symbol, decimals)
    pub async fn get_token_metadata(
        &self,
        token_address: &str,
    ) -> Result<TokenMetadata, Box<dyn std::error::Error + Send + Sync>> {
        let address: Address = token_address.parse()?;
        let contract = ERC20::new(address, self.provider.clone());

        let name = contract.name().call().await.unwrap_or_else(|_| "Unknown".to_string());
        let symbol = contract.symbol().call().await.unwrap_or_else(|_| "???".to_string());
        let decimals = contract.decimals().call().await.unwrap_or(18);
        let total_supply = contract.total_supply().call().await.unwrap_or(U256::zero());

        Ok(TokenMetadata {
            name,
            symbol,
            decimals,
            total_supply,
        })
    }

    /// Find liquidity pair for a token
    pub async fn find_pair(
        &self,
        token_address: &str,
        quote_token_address: &str,
        factory_address: &str,
    ) -> Result<Option<Address>, Box<dyn std::error::Error + Send + Sync>> {
        let factory: Address = factory_address.parse()?;
        let token: Address = token_address.parse()?;
        let quote: Address = quote_token_address.parse()?;

        let factory_contract = UniswapV2Factory::new(factory, self.provider.clone());
        let pair_address = factory_contract.get_pair(token, quote).call().await?;

        // Check if pair exists (non-zero address)
        if pair_address == Address::zero() {
            Ok(None)
        } else {
            Ok(Some(pair_address))
        }
    }

    /// Get pair reserves and calculate token price
    pub async fn get_pair_data(
        &self,
        pair_address: Address,
        token_address: &str,
    ) -> Result<PairData, Box<dyn std::error::Error + Send + Sync>> {
        let pair_contract = UniswapV2Pair::new(pair_address, self.provider.clone());
        let token: Address = token_address.parse()?;

        // Get reserves
        let (reserve0, reserve1, _) = pair_contract.get_reserves().call().await?;

        // Get token addresses
        let token0 = pair_contract.token_0().call().await?;
        let _token1 = pair_contract.token_1().call().await?;

        // Determine which reserve is our token
        let (token_reserve, quote_reserve) = if token0 == token {
            (reserve0, reserve1)
        } else {
            (reserve1, reserve0)
        };

        Ok(PairData {
            token_reserve: U256::from(token_reserve),
            quote_reserve: U256::from(quote_reserve),
            pair_address,
        })
    }

    /// Calculate token price in USD
    pub async fn calculate_token_price(
        &self,
        token_address: &str,
        factory_address: &str,
        wbnb_address: &str,
        busd_address: &str,
    ) -> Result<TokenPrice, Box<dyn std::error::Error + Send + Sync>> {
        // First, try to find token/BUSD pair (direct USD price)
        if let Some(pair_address) = self
            .find_pair(token_address, busd_address, factory_address)
            .await?
        {
            let pair_data = self.get_pair_data(pair_address, token_address).await?;
            let token_metadata = self.get_token_metadata(token_address).await?;

            // Price = quote_reserve / token_reserve
            let price = calculate_price(
                pair_data.token_reserve,
                pair_data.quote_reserve,
                token_metadata.decimals,
                18, // BUSD decimals
            );

            let liquidity_usd = calculate_liquidity(
                pair_data.quote_reserve,
                18, // BUSD decimals
            );

            return Ok(TokenPrice {
                price_usd: price,
                liquidity_usd,
                pair_address: Some(pair_address),
            });
        }

        // If no BUSD pair, try WBNB pair and convert to USD
        if let Some(pair_address) = self
            .find_pair(token_address, wbnb_address, factory_address)
            .await?
        {
            let pair_data = self.get_pair_data(pair_address, token_address).await?;
            let token_metadata = self.get_token_metadata(token_address).await?;

            // Get BNB price in BUSD
            let bnb_price = self.get_bnb_price(factory_address, wbnb_address, busd_address).await?;

            // Price in BNB
            let price_in_bnb = calculate_price(
                pair_data.token_reserve,
                pair_data.quote_reserve,
                token_metadata.decimals,
                18, // WBNB decimals
            );

            // Convert to USD
            let price_usd = price_in_bnb * bnb_price;

            let liquidity_bnb = calculate_liquidity(
                pair_data.quote_reserve,
                18, // WBNB decimals
            );
            let liquidity_usd = liquidity_bnb * bnb_price;

            return Ok(TokenPrice {
                price_usd,
                liquidity_usd,
                pair_address: Some(pair_address),
            });
        }

        // No pair found
        Err("No liquidity pair found".into())
    }

    /// Get BNB price in USD from WBNB/BUSD pair
    async fn get_bnb_price(
        &self,
        factory_address: &str,
        wbnb_address: &str,
        busd_address: &str,
    ) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
        let pair_address = self
            .find_pair(wbnb_address, busd_address, factory_address)
            .await?
            .ok_or("WBNB/BUSD pair not found")?;

        let pair_data = self.get_pair_data(pair_address, wbnb_address).await?;

        // BNB price = BUSD_reserve / WBNB_reserve
        let price = calculate_price(
            pair_data.token_reserve,
            pair_data.quote_reserve,
            18, // WBNB decimals
            18, // BUSD decimals
        );

        Ok(price)
    }
}

// Helper function to calculate price from reserves
fn calculate_price(
    token_reserve: U256,
    quote_reserve: U256,
    token_decimals: u8,
    quote_decimals: u8,
) -> f64 {
    let token_reserve_f64 = token_reserve.as_u128() as f64 / 10f64.powi(token_decimals as i32);
    let quote_reserve_f64 = quote_reserve.as_u128() as f64 / 10f64.powi(quote_decimals as i32);

    if token_reserve_f64 == 0.0 {
        return 0.0;
    }

    quote_reserve_f64 / token_reserve_f64
}

// Helper function to calculate liquidity (2x quote reserve)
fn calculate_liquidity(quote_reserve: U256, quote_decimals: u8) -> f64 {
    let quote_reserve_f64 = quote_reserve.as_u128() as f64 / 10f64.powi(quote_decimals as i32);
    quote_reserve_f64 * 2.0 // Total liquidity is 2x one side
}

#[derive(Debug)]
pub struct TokenMetadata {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub total_supply: U256,
}

#[derive(Debug)]
pub struct PairData {
    pub token_reserve: U256,
    pub quote_reserve: U256,
    pub pair_address: Address,
}

#[derive(Debug)]
pub struct TokenPrice {
    pub price_usd: f64,
    pub liquidity_usd: f64,
    pub pair_address: Option<Address>,
}
