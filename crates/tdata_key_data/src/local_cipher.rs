//! AES-256-IGE with the tdata `key128` derivation.
//!
//! Source of truth: `MTP::aesEncryptLocal` / `MTP::aesDecryptLocal` in
//! `mtproto/mtproto_auth_key.h`, which derive the IGE key and IV from the
//! first 16 bytes of the encrypted blob through `prepareAES_oldmtp`.
//!
//! Both directions derive with `send = false`: the same 128-bit header is the
//! key material and the IGE initialisation vector, so decryption needs nothing
//! but the blob itself.

use crypto_aes_ige::{aes_ige_decrypt, aes_ige_encrypt, IgeError};
use crypto_kdf::prepare_aes_oldmtp;

use crate::local_key::LocalKey;

/// The 128-bit key/IV header that prefixes every encrypted block.
pub type Key128 = [u8; 16];

/// Derives the AES-256 key and 32-byte IGE IV for one block.
fn local_aes(key: &LocalKey, key128: &Key128) -> ([u8; 32], [u8; 32]) {
    prepare_aes_oldmtp(key.as_bytes(), key128, false)
}

/// `aesEncryptLocal` — encrypts `src` into a fresh buffer.
///
/// `src.len()` must be a multiple of 16, which `PrepareEncrypted` guarantees by
/// padding; a short or unaligned input is reported as [`IgeError::InvalidLength`].
pub fn aes_encrypt_local(
    src: &[u8],
    key: &LocalKey,
    key128: &Key128,
) -> Result<Vec<u8>, IgeError> {
    let (aes_key, aes_iv) = local_aes(key, key128);
    let mut dst = src.to_vec();
    aes_ige_encrypt(&mut dst, &aes_key, &aes_iv)?;
    Ok(dst)
}

/// `aesDecryptLocal` — decrypts `src` into a fresh buffer.
///
/// Same length rule as [`aes_encrypt_local`]; a caller filtering blocks out of
/// a file has already checked `len > 16 && len % 16 == 0`.
pub fn aes_decrypt_local(
    src: &[u8],
    key: &LocalKey,
    key128: &Key128,
) -> Result<Vec<u8>, IgeError> {
    let (aes_key, aes_iv) = local_aes(key, key128);
    let mut dst = src.to_vec();
    aes_ige_decrypt(&mut dst, &aes_key, &aes_iv)?;
    Ok(dst)
}