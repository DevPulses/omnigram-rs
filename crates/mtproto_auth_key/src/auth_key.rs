//! MTProto AuthKey management matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/mtproto/mtproto_auth_key.{h,cpp}`).

use sha1::{Digest, Sha1};
use crypto_kdf::{prepare_aes_oldmtp, prepare_aes};

pub const AUTH_KEY_SIZE: usize = 256;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthKeyType {
    Generated,
    Temporary,
    ReadFromFile,
    Local,
}

#[derive(Clone, PartialEq, Eq)]
pub struct AuthKey {
    type_: AuthKeyType,
    dc_id: i32,
    key: [u8; AUTH_KEY_SIZE],
    key_id: u64,
    expires_at: i32,
}

impl AuthKey {
    pub fn new(type_: AuthKeyType, dc_id: i32, key_bytes: &[u8]) -> Self {
        let mut key = [0u8; AUTH_KEY_SIZE];
        Self::fill_data(&mut key, key_bytes);
        let key_id = Self::count_key_id(&key);
        Self {
            type_,
            dc_id,
            key,
            key_id,
            expires_at: 0,
        }
    }

    pub fn from_key(key: [u8; AUTH_KEY_SIZE]) -> Self {
        let key_id = Self::count_key_id(&key);
        Self {
            type_: AuthKeyType::Local,
            dc_id: 0,
            key,
            key_id,
            expires_at: 0,
        }
    }

    pub fn type_(&self) -> AuthKeyType {
        self.type_
    }

    pub fn dc_id(&self) -> i32 {
        self.dc_id
    }

    pub fn key_id(&self) -> u64 {
        self.key_id
    }

    pub fn data(&self) -> &[u8; AUTH_KEY_SIZE] {
        &self.key
    }

    pub fn expires_at(&self) -> i32 {
        self.expires_at
    }

    pub fn set_expires_at(&mut self, expires_at: i32) {
        assert_eq!(self.type_, AuthKeyType::Temporary);
        self.expires_at = expires_at;
    }

    pub fn fill_data(dst: &mut [u8; AUTH_KEY_SIZE], src: &[u8]) {
        assert!(src.len() <= AUTH_KEY_SIZE);
        if src.len() < AUTH_KEY_SIZE {
            let offset = AUTH_KEY_SIZE - src.len();
            dst[..offset].fill(0);
            dst[offset..].copy_from_slice(src);
        } else {
            dst.copy_from_slice(src);
        }
    }

    pub fn count_key_id(key: &[u8; AUTH_KEY_SIZE]) -> u64 {
        let mut hasher = Sha1::new();
        hasher.update(key);
        let hash = hasher.finalize();
        // Lower 64 bits = 8 bytes of 20-byte SHA1 hash (bytes 12..20)
        let mut id_bytes = [0u8; 8];
        id_bytes.copy_from_slice(&hash[12..20]);
        u64::from_le_bytes(id_bytes)
    }

    /// MTProto 2.0 AES key and IV derivation
    pub fn prepare_aes(&self, msg_key: &[u8; 16], send: bool) -> ([u8; 32], [u8; 32]) {
        prepare_aes(&self.key, msg_key, send)
    }

    /// MTProto 1.0 (legacy) AES key and IV derivation
    pub fn prepare_aes_oldmtp(&self, msg_key: &[u8; 16], send: bool) -> ([u8; 32], [u8; 32]) {
        prepare_aes_oldmtp(&self.key, msg_key, send)
    }

    pub fn part_for_msg_key(&self, send: bool) -> &[u8] {
        let offset = 88 + if send { 0 } else { 8 };
        &self.key[offset..offset + 32]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_key_id() {
        let mut key_bytes = [0u8; AUTH_KEY_SIZE];
        key_bytes[0] = 42;
        let auth_key = AuthKey::from_key(key_bytes);
        assert_ne!(auth_key.key_id(), 0);
    }

    #[test]
    fn test_fill_data() {
        let mut key = [0u8; AUTH_KEY_SIZE];
        let short = [1u8, 2, 3];
        AuthKey::fill_data(&mut key, &short);
        assert_eq!(&key[AUTH_KEY_SIZE - 3..], &[1, 2, 3]);
        assert_eq!(key[0], 0);
    }
}
