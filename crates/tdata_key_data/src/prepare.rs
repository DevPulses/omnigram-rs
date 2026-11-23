//! `PrepareEncrypted` — turns a block payload into the blob stored in a file.
//!
//! Source of truth: `PrepareEncrypted` in `storage_file_utilities.cpp`.
//!
//! The payload is padded with random bytes to a 16-byte multiple, its total
//! size goes into the leading `uint32`, and the first 16 bytes of `SHA1` over
//! the padded payload become the block header — both the `key128` input of the
//! AES-IGE derivation and its IV. The result is `header || ciphertext`.

use sha1::{Digest, Sha1};

use crate::descriptor::EncryptedDescriptor;
use crate::local_cipher::{aes_encrypt_local, Key128};
use crate::local_key::LocalKey;

/// Encrypts the payload, padding it with random bytes.
#[must_use]
pub fn prepare_encrypted(data: &mut EncryptedDescriptor, key: &LocalKey) -> Vec<u8> {
    prepare_encrypted_with(data, key, |padding| {
        rand::RngCore::fill_bytes(&mut rand::thread_rng(), padding);
    })
}

/// Encrypts the payload, padding it with whatever `fill` writes.
///
/// Splitting the padding out keeps the format testable: the padding is the
/// only part of a file that is not a function of the payload.
#[must_use]
pub fn prepare_encrypted_with(
    data: &mut EncryptedDescriptor,
    key: &LocalKey,
    mut fill: impl FnMut(&mut [u8]),
) -> Vec<u8> {
    let to_encrypt = data.data_mut();
    let size = to_encrypt.len() as u32;
    let mut full_size = to_encrypt.len();
    if full_size & 0x0F != 0 {
        full_size += 0x10 - (full_size & 0x0F);
        to_encrypt.resize(full_size, 0);
        fill(&mut to_encrypt[size as usize..full_size]);
    }
    to_encrypt[..crate::descriptor::SIZE_PREFIX].copy_from_slice(&size.to_le_bytes());

    let digest = Sha1::digest(&to_encrypt);
    let mut key128: Key128 = [0u8; 16];
    key128.copy_from_slice(&digest[..16]);

    let encrypted = aes_encrypt_local(to_encrypt, key, &key128)
        .expect("the payload is padded to a whole number of 16-byte blocks");

    let mut result = Vec::with_capacity(16 + encrypted.len());
    result.extend_from_slice(&key128);
    result.extend_from_slice(&encrypted);
    result
}