//! `DecryptLocal` — checks a stored blob and opens its block payload.
//!
//! Source of truth: `DecryptLocal` in `storage_file_utilities.cpp`.
//!
//! The 16-byte header is the AES-IGE key material *and* the IV, and the first
//! 16 bytes of `SHA1` over the plaintext must reproduce it — that is what tells
//! a wrong passcode from a damaged file. The plaintext then starts with its own
//! total size, which is validated against the decrypted length.

use sha1::{Digest, Sha1};

use crate::descriptor::{DataReader, SIZE_PREFIX};
use crate::local_cipher::{aes_decrypt_local, Key128};
use crate::local_key::LocalKey;

/// A successfully decrypted payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decrypted {
    /// Plaintext bytes, `dataLen` long and including the size prefix.
    pub data: Vec<u8>,
    /// Stream position of the first block: right after the size prefix.
    pub position: usize,
}

impl Decrypted {
    /// A reader over the blocks, positioned at the first one.
    #[must_use]
    pub fn reader(&self) -> DataReader<'_> {
        DataReader::new(&self.data, self.position)
    }
}

/// Decrypts one block blob, rejecting everything `DecryptLocal` rejects.
#[must_use]
pub fn decrypt_local(encrypted: &[u8], key: &LocalKey) -> Option<Decrypted> {
    if encrypted.len() <= 16 || encrypted.len() & 0x0F != 0 {
        return None;
    }
    let full_len = encrypted.len() - 16;
    let key128: Key128 = encrypted[..16].try_into().ok()?;
    let decrypted = aes_decrypt_local(&encrypted[16..], key, &key128).ok()?;

    let digest = Sha1::digest(&decrypted);
    if digest[..16] != key128 {
        return None;
    }

    let data_len = u32::from_le_bytes(decrypted[..SIZE_PREFIX].try_into().ok()?) as usize;
    if data_len > decrypted.len() || data_len <= full_len - 16 || data_len < SIZE_PREFIX {
        return None;
    }

    let mut data = decrypted;
    data.truncate(data_len);
    Some(Decrypted {
        data,
        position: SIZE_PREFIX,
    })
}