use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

const NONCE_LEN: usize = 24;
const TAG_LEN: usize = 16;

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut bytes = [0u8; N];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

pub fn derive_key(password: &str, salt: &[u8]) -> AppResult<[u8; 32]> {
    let params = Params::new(64 * 1024, 3, 4, Some(32))
        .map_err(|error| AppError::Crypto(error.to_string()))?;
    let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let mut key = [0u8; 32];
    argon
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|error| AppError::Crypto(error.to_string()))?;
    Ok(key)
}

pub fn fingerprint(key: &[u8; 32]) -> String {
    to_hex(&Sha256::digest(key))
}

pub fn encrypt(key: &[u8; 32], plaintext: &str) -> AppResult<String> {
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|error| AppError::Crypto(error.to_string()))?;
    let nonce = random_bytes::<NONCE_LEN>();
    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), plaintext.as_bytes())
        .map_err(|error| AppError::Crypto(error.to_string()))?;

    let mut payload = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    payload.extend_from_slice(&nonce);
    payload.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(payload))
}

pub fn decrypt(key: &[u8; 32], encoded: &str) -> AppResult<String> {
    let payload = STANDARD
        .decode(encoded)
        .map_err(|_| AppError::Crypto("密文格式无效".to_string()))?;

    if payload.len() < NONCE_LEN + TAG_LEN {
        return Err(AppError::Crypto("密文长度无效".to_string()));
    }

    let (nonce, ciphertext) = payload.split_at(NONCE_LEN);
    let cipher = XChaCha20Poly1305::new_from_slice(key)
        .map_err(|error| AppError::Crypto(error.to_string()))?;
    let plaintext = cipher
        .decrypt(XNonce::from_slice(nonce), ciphertext)
        .map_err(|_| AppError::Crypto("解密失败，密钥可能不匹配".to_string()))?;

    String::from_utf8(plaintext).map_err(|_| AppError::Crypto("明文不是有效的 UTF-8".to_string()))
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn from_hex(value: &str) -> AppResult<Vec<u8>> {
    if value.len() % 2 != 0 {
        return Err(AppError::Crypto("十六进制格式无效".to_string()));
    }

    (0..value.len())
        .step_by(2)
        .map(|index| {
            u8::from_str_radix(&value[index..index + 2], 16)
                .map_err(|_| AppError::Crypto("十六进制格式无效".to_string()))
        })
        .collect()
}
