//! The 256-byte local key and its two password derivations.
//!
//! Source of truth: `CreateLocalKey` / `CreateLegacyLocalKey` in
//! `storage_file_utilities.cpp` and `LocalEncrypt*` in `config.h`.
//!
//! The key is `MTP::AuthKey::Data` — 256 bytes — only used as the input of
//! `prepareAES_oldmtp` (see [`crate::local_cipher`]), never as a file key.

use pbkdf2::pbkdf2_hmac;
use sha1::Sha1;
use sha2::{Digest, Sha512};

/// Size of `MTP::AuthKey::Data` — the local key that wraps tdata files.
pub const LOCAL_KEY_SIZE: usize = 256;

/// `kStrongIterationsCount` — PBKDF2 rounds for the current local key when a
/// passcode is set. An empty passcode uses a single round instead.
pub const STRONG_ITERATIONS_COUNT: u32 = 100_000;

/// `LocalEncryptIterCount` — PBKDF2 rounds of the legacy (pre-SHA512) key.
pub const LEGACY_ITERATIONS_COUNT: u32 = 4_000;

/// `LocalEncryptNoPwdIterCount` — legacy rounds when no passcode is set.
pub const LEGACY_NO_PASSWORD_ITERATIONS_COUNT: u32 = 4;

/// `LocalEncryptSaltSize` — the legacy salt length is fixed at 256 bits.
pub const LEGACY_SALT_SIZE: usize = 32;

/// The 256-byte local key wrapping every account file.
#[derive(Clone, PartialEq, Eq)]
pub struct LocalKey([u8; LOCAL_KEY_SIZE]);

impl LocalKey {
    /// Builds a key from raw bytes, e.g. one read back from a derived store.
    #[must_use]
    pub fn from_bytes(bytes: [u8; LOCAL_KEY_SIZE]) -> Self {
        Self(bytes)
    }

    /// Builds a key from a slice of exactly [`LOCAL_KEY_SIZE`] bytes.
    #[must_use]
    pub fn from_slice(bytes: &[u8]) -> Option<Self> {
        let bytes: [u8; LOCAL_KEY_SIZE] = bytes.try_into().ok()?;
        Some(Self(bytes))
    }

    /// The raw key material.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; LOCAL_KEY_SIZE] {
        &self.0
    }
}

impl std::fmt::Debug for LocalKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("LocalKey(..)")
    }
}

/// `CreateLocalKey` — the current key derivation.
///
/// The passcode is pre-hashed as `SHA512(salt + passcode + salt)`, then run
/// through PBKDF2-HMAC-SHA512 over `salt` into 256 bytes. An empty passcode
/// takes one round so that a key without a password stays fast.
#[must_use]
pub fn create_local_key(passcode: &[u8], salt: &[u8]) -> LocalKey {
    let mut hash = Sha512::new();
    hash.update(salt);
    hash.update(passcode);
    hash.update(salt);
    let hash = hash.finalize();

    let iterations = if passcode.is_empty() {
        1
    } else {
        STRONG_ITERATIONS_COUNT
    };
    let mut key = [0u8; LOCAL_KEY_SIZE];
    pbkdf2_hmac::<Sha512>(&hash, salt, iterations, &mut key);
    LocalKey(key)
}

/// `CreateLegacyLocalKey` — the pre-SHA512 key derivation still needed to open
/// tdata written by old versions: PBKDF2-HMAC-SHA1 over the passcode itself.
#[must_use]
pub fn create_legacy_local_key(passcode: &[u8], salt: &[u8]) -> LocalKey {
    let iterations = if passcode.is_empty() {
        LEGACY_NO_PASSWORD_ITERATIONS_COUNT
    } else {
        LEGACY_ITERATIONS_COUNT
    };
    let mut key = [0u8; LOCAL_KEY_SIZE];
    pbkdf2_hmac::<Sha1>(passcode, salt, iterations, &mut key);
    LocalKey(key)
}