//! AES-256-IGE mode implementation matching Telegram's OpenSSL AES_ige_encrypt.
//!
//! In IGE (Infinite Garble Extension) mode:
//! Encryption:
//!   c_i = AES_enc(p_i ^ c_{i-1}) ^ m_{i-1}
//!   where c_0 = iv_0, m_0 = iv_1 (iv is 32 bytes: iv_0 is first 16 bytes, iv_1 is second 16 bytes).
//!   m_i = p_i
//! Decryption:
//!   p_i = AES_dec(c_i ^ m_{i-1}) ^ c_{i-1}
//!   where c_0 = iv_0, m_0 = iv_1
//!   m_i = p_i

use aes::cipher::{BlockDecrypt, BlockEncrypt, KeyInit};
use aes::Aes256;

pub const BLOCK_SIZE: usize = 16;
pub const KEY_SIZE: usize = 32;
pub const IV_SIZE: usize = 32;

#[derive(Debug, PartialEq, Eq)]
pub enum IgeError {
    InvalidLength,
}

/// In-place encrypts `data` using AES-256 in IGE mode.
/// `key` is 32 bytes (256 bits).
/// `iv` is 32 bytes (iv_0: [0..16], iv_1: [16..32]).
pub fn aes_ige_encrypt(data: &mut [u8], key: &[u8; KEY_SIZE], iv: &[u8; IV_SIZE]) -> Result<(), IgeError> {
    if data.len() % BLOCK_SIZE != 0 {
        return Err(IgeError::InvalidLength);
    }

    let cipher = Aes256::new_from_slice(key).unwrap();

    let mut iv1 = [0u8; BLOCK_SIZE];
    let mut iv2 = [0u8; BLOCK_SIZE];
    iv1.copy_from_slice(&iv[..16]);
    iv2.copy_from_slice(&iv[16..]);

    for chunk in data.chunks_exact_mut(BLOCK_SIZE) {
        let mut block = aes::Block::default();
        // block = chunk ^ iv1
        for i in 0..BLOCK_SIZE {
            block[i] = chunk[i] ^ iv1[i];
        }

        // AES encrypt
        cipher.encrypt_block(&mut block);

        // block ^= iv2
        for i in 0..BLOCK_SIZE {
            block[i] ^= iv2[i];
        }

        // iv1 = block (new ciphertext block c_i)
        // iv2 = chunk (old plaintext block p_i)
        iv2.copy_from_slice(chunk);
        iv1.copy_from_slice(&block);
        chunk.copy_from_slice(&block);
    }

    Ok(())
}

/// In-place decrypts `data` using AES-256 in IGE mode.
/// `key` is 32 bytes (256 bits).
/// `iv` is 32 bytes (iv_0: [0..16], iv_1: [16..32]).
pub fn aes_ige_decrypt(data: &mut [u8], key: &[u8; KEY_SIZE], iv: &[u8; IV_SIZE]) -> Result<(), IgeError> {
    if data.len() % BLOCK_SIZE != 0 {
        return Err(IgeError::InvalidLength);
    }

    let cipher = Aes256::new_from_slice(key).unwrap();

    let mut iv1 = [0u8; BLOCK_SIZE];
    let mut iv2 = [0u8; BLOCK_SIZE];
    iv1.copy_from_slice(&iv[..16]);
    iv2.copy_from_slice(&iv[16..]);

    for chunk in data.chunks_exact_mut(BLOCK_SIZE) {
        let mut block = aes::Block::default();
        // block = chunk ^ iv2
        for i in 0..BLOCK_SIZE {
            block[i] = chunk[i] ^ iv2[i];
        }

        // AES decrypt
        cipher.decrypt_block(&mut block);

        // block ^= iv1
        for i in 0..BLOCK_SIZE {
            block[i] ^= iv1[i];
        }

        // iv1 = chunk (old ciphertext block c_i)
        // iv2 = block (new plaintext block p_i)
        iv1.copy_from_slice(chunk);
        iv2.copy_from_slice(&block);
        chunk.copy_from_slice(&block);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_256_ige_openssl_oracle_vector() {
        let key = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
        let iv = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
        let pt = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
        // Verified directly against OpenSSL 3.x AES_ige_encrypt
        let expected_ct = hex::decode("e28112a53e5c89c7b1ea8071c133699fd4e86b26c4bac9fc5af1ab8ce27a44a4").unwrap();

        let mut key_arr = [0u8; 32];
        key_arr.copy_from_slice(&key);
        let mut iv_arr = [0u8; 32];
        iv_arr.copy_from_slice(&iv);

        let mut data = pt.clone();
        aes_ige_encrypt(&mut data, &key_arr, &iv_arr).unwrap();
        assert_eq!(data, expected_ct);

        aes_ige_decrypt(&mut data, &key_arr, &iv_arr).unwrap();
        assert_eq!(data, pt);
    }

    #[test]
    fn test_roundtrip_arbitrary_blocks() {
        let key = [0x42u8; 32];
        let iv = [0x24u8; 32];
        let original: Vec<u8> = (0..64).map(|x| (x * 7 + 13) as u8).collect();
        let mut buf = original.clone();

        aes_ige_encrypt(&mut buf, &key, &iv).unwrap();
        assert_ne!(buf, original);

        aes_ige_decrypt(&mut buf, &key, &iv).unwrap();
        assert_eq!(buf, original);
    }

    #[test]
    fn test_invalid_length() {
        let key = [0u8; 32];
        let iv = [0u8; 32];
        let mut buf = vec![0u8; 15];
        assert_eq!(aes_ige_encrypt(&mut buf, &key, &iv), Err(IgeError::InvalidLength));
        assert_eq!(aes_ige_decrypt(&mut buf, &key, &iv), Err(IgeError::InvalidLength));
    }
}
