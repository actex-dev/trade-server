use std::collections::HashMap;

/// Configuration for blockchain RPC connections
pub struct BlockchainConfig {
    pub rpc_urls: HashMap<String, String>,
    pub dex_contracts: DexContracts,
    pub stable_tokens: HashMap<String, String>,
}

pub struct DexContracts {
    pub pancakeswap_v2_factory: String,
    pub pancakeswap_v2_router: String,
}

impl BlockchainConfig {
    pub fn new() -> Self {
        let mut rpc_urls = HashMap::new();
        let mut stable_tokens = HashMap::new();

        // BSC RPC endpoints (public)
        rpc_urls.insert(
            "bsc".to_string(),
            "https://bsc-dataseed.binance.org/".to_string(),
        );

        // Solana RPC (for future use)
        rpc_urls.insert(
            "solana".to_string(),
            "https://api.mainnet-beta.solana.com".to_string(),
        );

        // BSC stable tokens for price calculation
        stable_tokens.insert(
            "bsc_wbnb".to_string(),
            "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c".to_string(), // WBNB
        );
        stable_tokens.insert(
            "bsc_busd".to_string(),
            "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56".to_string(), // BUSD
        );
        stable_tokens.insert(
            "bsc_usdt".to_string(),
            "0x55d398326f99059fF775485246999027B3197955".to_string(), // USDT
        );

        Self {
            rpc_urls,
            dex_contracts: DexContracts {
                pancakeswap_v2_factory: "0xcA143Ce32Fe78f1f7019d7d551a6402fC5350c73"
                    .to_string(),
                pancakeswap_v2_router: "0x10ED43C718714eb63d5aA57B78B54704E256024E"
                    .to_string(),
            },
            stable_tokens,
        }
    }

    pub fn get_rpc_url(&self, chain_id: &str) -> Option<&String> {
        self.rpc_urls.get(chain_id)
    }

    pub fn get_wbnb_address(&self) -> &str {
        &self.stable_tokens["bsc_wbnb"]
    }

    pub fn get_busd_address(&self) -> &str {
        &self.stable_tokens["bsc_busd"]
    }
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self::new()
    }
}
