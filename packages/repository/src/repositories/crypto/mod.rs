use bip39::Mnemonic;
use data::{CryptoConfig, CryptoError, Wallet};
use hex;
use rand::Rng;
use sha2::{Digest, Sha256};

pub mod data;

#[allow(dead_code)]
pub trait CryptoRepositoryTrait {
    /// Create a new wallet with address, private key, and seed phrase
    fn create_wallet(&self) -> Result<Wallet, CryptoError>;
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct CryptoRepository {
    config: CryptoConfig,
}

#[allow(dead_code)]
impl CryptoRepository {
    pub fn new(config: CryptoConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self {
            config: CryptoConfig::default(),
        }
    }

    /// Get a reference to the repository's configuration
    pub fn config(&self) -> &CryptoConfig {
        &self.config
    }

    /// Generate a random private key (32 bytes)
    fn generate_private_key(&self) -> String {
        let mut rng = rand::thread_rng();
        let private_key: [u8; 32] = rng.gen();
        hex::encode(private_key)
    }

    /// Derive address from private key (simplified - in production use proper key derivation)
    fn derive_address(&self, private_key: &str) -> Result<String, CryptoError> {
        // This is a simplified version. In production, use proper elliptic curve cryptography
        // For Ethereum: use secp256k1, keccak256
        // For Bitcoin: use secp256k1, ripemd160, base58

        let mut hasher = Sha256::new();
        hasher.update(private_key.as_bytes());
        let result = hasher.finalize();

        // Simplified address format (0x + first 40 chars of hash)
        Ok(format!("0x{}", hex::encode(&result[..20])))
    }

    /// Generate mnemonic seed phrase
    fn generate_seed_phrase(&self) -> Result<String, CryptoError> {
        let mut rng = rand::thread_rng();
        let entropy: [u8; 32] = rng.gen();

        match Mnemonic::from_entropy(&entropy) {
            Ok(mnemonic) => Ok(mnemonic.to_string()),
            Err(e) => Err(CryptoError::WalletCreationError(format!(
                "Failed to generate mnemonic: {}",
                e
            ))),
        }
    }
}

impl CryptoRepositoryTrait for CryptoRepository {
    fn create_wallet(&self) -> Result<Wallet, CryptoError> {
        // Generate seed phrase
        let seed_phrase = self.generate_seed_phrase()?;

        // Generate private key
        let private_key = self.generate_private_key();

        // Derive address from private key
        let address = self.derive_address(&private_key)?;

        Ok(Wallet::new(address, private_key, seed_phrase))
    }
}
