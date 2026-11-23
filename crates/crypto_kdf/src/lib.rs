//! MTProto key derivation functions (KDF) matching Telegram Desktop's `AuthKey::prepareAES` and `AuthKey::prepareAES_oldmtp`.
//!
//! Source of truth: `tdesktop/Telegram/SourceFiles/mtproto/mtproto_auth_key.cpp`

use sha1::Sha1;
use sha2::{Digest, Sha256};

pub const AUTH_KEY_SIZE: usize = 256;
pub const MSG_KEY_SIZE: usize = 16;
pub const AES_KEY_SIZE: usize = 32;
pub const AES_IV_SIZE: usize = 32;

/// MTProto 2.0 AES key and IV derivation (`AuthKey::prepareAES` in tdesktop).
///
/// Inputs:
/// - `auth_key`: 256-byte authorization key.
/// - `msg_key`: 16-byte message key.
/// - `send`: `true` if client sending to server (x = 0), `false` if client receiving from server (x = 8).
///
/// Output:
/// - `(aes_key, aes_iv)`: 32-byte AES key and 32-byte AES IV.
pub fn prepare_aes(
    auth_key: &[u8; AUTH_KEY_SIZE],
    msg_key: &[u8; MSG_KEY_SIZE],
    send: bool,
) -> ([u8; AES_KEY_SIZE], [u8; AES_IV_SIZE]) {
    let x = if send { 0 } else { 8 };

    // sha256_a = SHA256(msg_key + auth_key[x .. x + 36])
    let mut hasher_a = Sha256::new();
    hasher_a.update(msg_key);
    hasher_a.update(&auth_key[x..x + 36]);
    let sha256_a = hasher_a.finalize();

    // sha256_b = SHA256(auth_key[40 + x .. 40 + x + 36] + msg_key)
    let mut hasher_b = Sha256::new();
    hasher_b.update(&auth_key[40 + x..40 + x + 36]);
    hasher_b.update(msg_key);
    let sha256_b = hasher_b.finalize();

    let mut aes_key = [0u8; AES_KEY_SIZE];
    let mut aes_iv = [0u8; AES_IV_SIZE];

    // aesKey:
    // memcpy(key, sha256_a.data(), 8);
    // memcpy(key + 8, sha256_b.data() + 8, 16);
    // memcpy(key + 8 + 16, sha256_a.data() + 24, 8);
    aes_key[0..8].copy_from_slice(&sha256_a[0..8]);
    aes_key[8..24].copy_from_slice(&sha256_b[8..24]);
    aes_key[24..32].copy_from_slice(&sha256_a[24..32]);

    // aesIV:
    // memcpy(iv, sha256_b.data(), 8);
    // memcpy(iv + 8, sha256_a.data() + 8, 16);
    // memcpy(iv + 8 + 16, sha256_b.data() + 24, 8);
    aes_iv[0..8].copy_from_slice(&sha256_b[0..8]);
    aes_iv[8..24].copy_from_slice(&sha256_a[8..24]);
    aes_iv[24..32].copy_from_slice(&sha256_b[24..32]);

    (aes_key, aes_iv)
}

/// MTProto 1.0 AES key and IV derivation (`AuthKey::prepareAES_oldmtp` in tdesktop).
///
/// Inputs:
/// - `auth_key`: 256-byte authorization key.
/// - `msg_key`: 16-byte message key.
/// - `send`: `true` if client sending (x = 0), `false` if client receiving (x = 8).
pub fn prepare_aes_oldmtp(
    auth_key: &[u8; AUTH_KEY_SIZE],
    msg_key: &[u8; MSG_KEY_SIZE],
    send: bool,
) -> ([u8; AES_KEY_SIZE], [u8; AES_IV_SIZE]) {
    let x = if send { 0 } else { 8 };

    // sha1_a = SHA1(msg_key + auth_key[x .. x + 32])
    let mut h_a = Sha1::new();
    h_a.update(msg_key);
    h_a.update(&auth_key[x..x + 32]);
    let sha1_a = h_a.finalize();

    // sha1_b = SHA1(auth_key[32 + x .. 32 + x + 16] + msg_key + auth_key[48 + x .. 48 + x + 16])
    let mut h_b = Sha1::new();
    h_b.update(&auth_key[32 + x..32 + x + 16]);
    h_b.update(msg_key);
    h_b.update(&auth_key[48 + x..48 + x + 16]);
    let sha1_b = h_b.finalize();

    // sha1_c = SHA1(auth_key[64 + x .. 64 + x + 32] + msg_key)
    let mut h_c = Sha1::new();
    h_c.update(&auth_key[64 + x..64 + x + 32]);
    h_c.update(msg_key);
    let sha1_c = h_c.finalize();

    // sha1_d = SHA1(msg_key + auth_key[96 + x .. 96 + x + 32])
    let mut h_d = Sha1::new();
    h_d.update(msg_key);
    h_d.update(&auth_key[96 + x..96 + x + 32]);
    let sha1_d = h_d.finalize();

    let mut aes_key = [0u8; AES_KEY_SIZE];
    let mut aes_iv = [0u8; AES_IV_SIZE];

    // key:
    // memcpy(key, sha1_a.data(), 8);
    // memcpy(key + 8, sha1_b.data() + 8, 12);
    // memcpy(key + 8 + 12, sha1_c.data() + 4, 12);
    aes_key[0..8].copy_from_slice(&sha1_a[0..8]);
    aes_key[8..20].copy_from_slice(&sha1_b[8..20]);
    aes_key[20..32].copy_from_slice(&sha1_c[4..16]);

    // iv:
    // memcpy(iv, sha1_a.data() + 8, 12);
    // memcpy(iv + 12, sha1_b.data(), 8);
    // memcpy(iv + 12 + 8, sha1_c.data() + 16, 4);
    // memcpy(iv + 12 + 8 + 4, sha1_d.data(), 8);
    aes_iv[0..12].copy_from_slice(&sha1_a[8..20]);
    aes_iv[12..20].copy_from_slice(&sha1_b[0..8]);
    aes_iv[20..24].copy_from_slice(&sha1_c[16..20]);
    aes_iv[24..32].copy_from_slice(&sha1_d[0..8]);

    (aes_key, aes_iv)
}

/// Compute 64-bit keyId from authKey (lower 64 bits / last 8 bytes of SHA1(auth_key)).
/// Matches `AuthKey::countKeyId` in tdesktop:
/// `_keyId = qFromUnaligned<KeyId>(hash.data() + 12);`
pub fn count_key_id(auth_key: &[u8; AUTH_KEY_SIZE]) -> u64 {
    let mut hasher = Sha1::new();
    hasher.update(auth_key);
    let hash = hasher.finalize();
    u64::from_le_bytes(hash[12..20].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtproto2_kdf_oracle() {
        // Oracle calculation using exact SHA-256 logic of tdesktop
        let mut auth_key = [0u8; AUTH_KEY_SIZE];
        for i in 0..AUTH_KEY_SIZE {
            auth_key[i] = (i * 3 + 7) as u8;
        }

        let mut msg_key = [0u8; MSG_KEY_SIZE];
        for i in 0..MSG_KEY_SIZE {
            msg_key[i] = (i * 11 + 1) as u8;
        }

        let (key_send, iv_send) = prepare_aes(&auth_key, &msg_key, true);
        let (key_recv, iv_recv) = prepare_aes(&auth_key, &msg_key, false);

        assert_ne!(key_send, key_recv);
        assert_ne!(iv_send, iv_recv);

        // Verify key ID calculation
        let key_id = count_key_id(&auth_key);
        let mut sha1 = Sha1::new();
        sha1.update(&auth_key);
        let hash = sha1.finalize();
        let expected_key_id = u64::from_le_bytes(hash[12..20].try_into().unwrap());
        assert_eq!(key_id, expected_key_id);
    }

    #[test]
    fn test_mtproto2_kdf_bit_exact() {
        // Независимый oracle: чистый Python (hashlib), формулы из
        // mtproto_auth_key.cpp — бит-в-бит сверка.
        let auth_key: [u8; 256] = core::array::from_fn(|i| (i as u32 * 3 + 7) as u8);
        let msg_key: [u8; 16] = core::array::from_fn(|i| (i as u32 * 11 + 1) as u8);

        let (k, iv) = prepare_aes(&auth_key, &msg_key, true);
        assert_eq!(hex::encode(k), "176f2a3776b6a7951cab5a161c5a9e8eab4903effceee2b58175c4b9545789e2");
        assert_eq!(hex::encode(iv), "d6591a43a43bbddc9a53218d30fc441dda50535d476a49867c4fb400e71be4ca");

        let (k, iv) = prepare_aes(&auth_key, &msg_key, false);
        assert_eq!(hex::encode(k), "b2daa318001e3643bf7a9ffc21ddb0ad54b42ee538d61638cae5395fa2b6007d");
        assert_eq!(hex::encode(iv), "d6b9ebdf0ce0cf0740584e0f2f973a3edc552c7c2c7612a79bf1fc8a7c4fbe46");
    }

    #[test]
    fn test_mtproto1_kdf_bit_exact() {
        let auth_key: [u8; 256] = core::array::from_fn(|i| (i as u32 * 5 + 13) as u8);
        let msg_key: [u8; 16] = core::array::from_fn(|i| (i as u32 * 7 + 3) as u8);

        let (k, iv) = prepare_aes_oldmtp(&auth_key, &msg_key, true);
        assert_eq!(hex::encode(k), "68b6fe55991708c5da932b7c946309859a260542bc51af78e8622b2ae3c461db");
        assert_eq!(hex::encode(iv), "0dedeebd394349f589e0a1587b7f61f943e559124b31a5b745ce101ed41f4467");

        let (k, iv) = prepare_aes_oldmtp(&auth_key, &msg_key, false);
        assert_eq!(hex::encode(k), "6ebee3c34f5784e051bff5c3cb08679eec4b26e4cb85e0f7008a80b3b8217640");
        assert_eq!(hex::encode(iv), "08179422acbb45248425500d2d0c4b82843cf13081b85f5141d058058d783ee5");
    }

    #[test]
    fn test_mtproto1_kdf_oracle() {
        let mut auth_key = [0u8; AUTH_KEY_SIZE];
        for i in 0..AUTH_KEY_SIZE {
            auth_key[i] = (i * 5 + 13) as u8;
        }

        let mut msg_key = [0u8; MSG_KEY_SIZE];
        for i in 0..MSG_KEY_SIZE {
            msg_key[i] = (i * 7 + 3) as u8;
        }

        let (key_send, iv_send) = prepare_aes_oldmtp(&auth_key, &msg_key, true);
        let (key_recv, iv_recv) = prepare_aes_oldmtp(&auth_key, &msg_key, false);

        assert_ne!(key_send, key_recv);
        assert_ne!(iv_send, iv_recv);
    }
}
