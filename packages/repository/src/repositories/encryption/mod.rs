use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::password_hash::rand_core::RngCore;
use data::{ArgonConfig, JwtConfig, EncryptionError, TokenParams};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation, Algorithm};
use base64::engine::general_purpose::{URL_SAFE_NO_PAD};
use base64::Engine;
use aes_gcm::{Aes256Gcm, aead::{Aead, KeyInit}, Nonce};
use sha2::{Sha256, Digest};

pub mod data;

#[allow(dead_code)]
pub trait EncryptionRepositoryTrait {
  fn hash_password(&self, plain: &str) -> Result<String, EncryptionError>;
  fn verify_password(&self, hash: &str, plain: &str) -> Result<bool, EncryptionError>;

  fn encrypt_data(&self, data: &str) -> Result<String, EncryptionError>;
  fn decrypt_data(&self, encrypted_data: &str) -> Result<String, EncryptionError>;
  
  fn create_token<T: serde::Serialize>(&self, payload: T, token_type: TokenParams) -> Result<String, EncryptionError>;
  fn decode_token(&self, token_string: &str, token_type: TokenParams) -> Result<serde_json::Value, EncryptionError>;
  fn create_code(&self, length: usize) -> String;
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct EncryptionRepository {
  argon: Argon2<'static>,
  argon_cfg: ArgonConfig,
  jwt_cfg: JwtConfig,
  encoding_key: EncodingKey,
  decoding_key: DecodingKey,
}


#[allow(dead_code)]
impl EncryptionRepository {
  pub fn new(argon_cfg: ArgonConfig, jwt_cfg: JwtConfig) -> Self {
    let argon = Argon2::default();
    let encoding_key = EncodingKey::from_secret(jwt_cfg.secret.as_bytes());
    let decoding_key = DecodingKey::from_secret(jwt_cfg.secret.as_bytes());
    
    Self {
      argon,
      argon_cfg,
      jwt_cfg,
      encoding_key,
      decoding_key,
    }
  }

  pub fn default() -> Self {
    let argon = Argon2::default();
    let jwt_cfg = JwtConfig {
      secret: "default_secret_key".to_string(),
      expiry_seconds: 3600,
    };
    let encoding_key = EncodingKey::from_secret(jwt_cfg.secret.as_bytes());
    let decoding_key = DecodingKey::from_secret(jwt_cfg.secret.as_bytes());
    
    Self {
      argon,
      argon_cfg: ArgonConfig {
        t_cost: 2,
        m_cost_kib: 65536,
        p_cost: 1,
      },
      jwt_cfg,
      encoding_key,
      decoding_key,
    }
  }
}

impl EncryptionRepositoryTrait for EncryptionRepository {
  fn hash_password(&self, plain: &str) -> Result<String, EncryptionError> {
    let salt = SaltString::generate(&mut OsRng);
    
    match self.argon.hash_password(plain.as_bytes(), &salt) {
      Ok(hash) => Ok(hash.to_string()),
      Err(e) => Err(EncryptionError::HashError(e.to_string())),
    }
  }

  fn verify_password(&self, hash: &str, plain: &str) -> Result<bool, EncryptionError> {
    let parsed_hash = match PasswordHash::new(hash) {
      Ok(hash) => hash,
      Err(e) => return Err(EncryptionError::VerifyError(e.to_string())),
    };

    match self.argon.verify_password(plain.as_bytes(), &parsed_hash) {
      Ok(_) => Ok(true),
      Err(_) => Ok(false),
    }
  }

  fn encrypt_data(&self, data: &str) -> Result<String, EncryptionError> {
    // Derive 256-bit key from repository secret
    let key_bytes = Sha256::digest(self.jwt_cfg.secret.as_bytes());
    let cipher = match Aes256Gcm::new_from_slice(&key_bytes) {
      Ok(c) => c,
      Err(e) => return Err(EncryptionError::JwtError(e.to_string())),
    };

    // Generate random 96-bit nonce
    let mut nonce_bytes = [0u8; 12];
    let mut rng = OsRng;
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt and encode as URL-safe base64: nonce || ciphertext
    let ciphertext = match cipher.encrypt(nonce, data.as_bytes()) {
      Ok(ct) => ct,
      Err(e) => return Err(EncryptionError::JwtError(e.to_string())),
    };

    let mut out = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(URL_SAFE_NO_PAD.encode(out))
  }

  fn decrypt_data(&self, encrypted_data: &str) -> Result<String, EncryptionError> {
    // Derive 256-bit key from repository secret
    let key_bytes = Sha256::digest(self.jwt_cfg.secret.as_bytes());
    let cipher = match Aes256Gcm::new_from_slice(&key_bytes) {
      Ok(c) => c,
      Err(e) => return Err(EncryptionError::JwtError(e.to_string())),
    };

    // Decode URL-safe base64 and split into nonce || ciphertext
    let raw = match URL_SAFE_NO_PAD.decode(encrypted_data) {
      Ok(r) => r,
      Err(e) => return Err(EncryptionError::JwtError(e.to_string())),
    };
    if raw.len() < 12 {
      return Err(EncryptionError::JwtError("Invalid encrypted data format".to_string()));
    }
    let (nonce_bytes, ciphertext) = raw.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = match cipher.decrypt(nonce, ciphertext) {
      Ok(pt) => pt,
      Err(e) => return Err(EncryptionError::JwtError(e.to_string())),
    };
    match String::from_utf8(plaintext) {
      Ok(s) => Ok(s),
      Err(e) => Err(EncryptionError::JwtError(e.to_string())),
    }
  }

  fn create_token<T: serde::Serialize>(&self, payload: T, token_type: TokenParams) -> Result<String, EncryptionError> {
    // Encode payload as a JSON string within claims `sub`
    let claims = match data::Claims::new_text(&payload, token_type.expiry_seconds) {
      Ok(claims) => claims,
      Err(e) => return Err(EncryptionError::JwtError(e.to_string())),
    };

    let encoding_key = EncodingKey::from_secret(token_type.key.as_bytes());
    
    match encode(&Header::default(), &claims, &encoding_key) {
      Ok(token) => Ok(token),
      Err(e) => Err(EncryptionError::JwtError(e.to_string())),
    }
  }

  fn decode_token(&self, token_string: &str, token_type: TokenParams) -> Result<serde_json::Value, EncryptionError> {
    // Normalize token (trim whitespace and surrounding quotes)
    let token = token_string.trim().trim_matches('"');

    let decoding_key = DecodingKey::from_secret(token_type.key.as_bytes());
    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::HS256, Algorithm::HS384, Algorithm::HS512];

    match decode::<serde_json::Value>(token, &decoding_key, &validation) {
      Ok(data) => Ok(data.claims),
      Err(e) => {
        tracing::info!("decode_token error: {}", e);
        Err(EncryptionError::JwtError(e.to_string()))
      }
    }
  }

  fn create_code(&self, length: usize) -> String {
    // Cryptographically secure random numeric code generation using OS RNG.
    // Uses rejection sampling to avoid modulo bias: accept bytes < 250 so 250 % 10 == 0.
    // Produces a string of digits [0-9] with uniform distribution.
    let mut code = String::with_capacity(length);
    let mut rng = OsRng;
    let mut buf = [0u8; 32];
    while code.len() < length {
      rng.fill_bytes(&mut buf);
      for &b in &buf {
        if code.len() >= length { break; }
        if b < 250 {
          let digit = (b % 10) as u8;
          code.push((b'0' + digit) as char);
        }
      }
    }
    code
  }
}