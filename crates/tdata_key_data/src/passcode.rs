//! The passcode KDF record and the memory-hard wrap around the local key.
//!
//! Source of truth: `PasscodeKdf`, `DefaultPasscodeKdf`, `CreatePasscodeKey`
//! and the `kPasscode*` constants in `storage_file_utilities.cpp`.
//!
//! Every wrap records its own family, cost numbers and salt, so changing the
//! defaults only affects wraps written afterwards; existing wraps are opened
//! with the numbers stored beside them.

use argon2::{Algorithm, Argon2, Params, Version};

use crate::local_key::{LocalKey, LOCAL_KEY_SIZE};

/// `kPasscodeKdfArgon2id` — memory-hard family, preferred.
pub const KDF_ARGON2ID: u32 = 1;
/// `kPasscodeKdfScrypt` — fallback family for builds without Argon2id.
pub const KDF_SCRYPT: u32 = 2;

/// `kPasscodeSaltMinSize` — a shorter salt makes a wrap unusable.
pub const SALT_MIN_SIZE: usize = 8;

/// `kPasscodeArgon2MemoryKiB` — 64 MiB, half of the 128 MiB a 32-bit address
/// space may not find contiguous.
pub const ARGON2_MEMORY_KIB: u32 = 65_536;
/// `kPasscodeArgon2Lanes` — one lane, so one thread fills the memory.
pub const ARGON2_LANES: u32 = 1;

/// `kPasscodeArgon2Time` — passes over the memory.
#[cfg(all(windows, target_arch = "x86"))]
pub const ARGON2_TIME: u32 = 4;
/// `kPasscodeArgon2Time` — passes over the memory.
#[cfg(not(all(windows, target_arch = "x86")))]
pub const ARGON2_TIME: u32 = 8;

/// `kPasscodeScryptN` — CPU/memory cost of the fallback family.
#[cfg(all(windows, target_arch = "x86"))]
pub const SCRYPT_N: u32 = 65_536;
/// `kPasscodeScryptN` — CPU/memory cost of the fallback family.
#[cfg(not(all(windows, target_arch = "x86")))]
pub const SCRYPT_N: u32 = 131_072;

/// `kPasscodeScryptR` — block size parameter.
pub const SCRYPT_R: u32 = 8;
/// `kPasscodeScryptP` — parallelization parameter.
pub const SCRYPT_P: u32 = 1;

/// `kPasscodeArgon2MaxMemoryKiB` — read ceiling, above the shipped cost so
/// later builds may raise the write cost without making files unreadable.
pub const ARGON2_MAX_MEMORY_KIB: u32 = 262_144;
/// `kPasscodeArgon2MaxTime` — read ceiling for the pass count.
pub const ARGON2_MAX_TIME: u32 = 64;
/// `kPasscodeArgon2MaxLanes` — read ceiling for the lane count.
pub const ARGON2_MAX_LANES: u32 = 16;

/// `kPasscodeScryptMaxMem` — 192 MiB cap OpenSSL's scrypt refuses to cross.
pub const SCRYPT_MAX_MEM: u64 = 192 * 1024 * 1024;

/// One recorded passcode wrap: which family, with which cost numbers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PasscodeKdf {
    pub kind: u32,
    pub memory: u32,
    pub time: u32,
    pub parallel: u32,
}

impl PasscodeKdf {
    /// Whether the cost numbers make sense for the recorded family.
    #[must_use]
    pub fn valid(&self) -> bool {
        if self.time == 0 || self.parallel == 0 {
            false
        } else if self.kind == KDF_ARGON2ID {
            self.memory >= 8 * self.parallel
        } else if self.kind == KDF_SCRYPT {
            self.memory >= 2 && (self.memory & (self.memory - 1)) == 0
        } else {
            false
        }
    }

    /// Whether the cost numbers stay under the read ceilings.
    #[must_use]
    pub fn cost_within_limits(&self) -> bool {
        self.kind != KDF_ARGON2ID
            || (self.memory <= ARGON2_MAX_MEMORY_KIB
                && self.time <= ARGON2_MAX_TIME
                && self.parallel <= ARGON2_MAX_LANES)
    }
}

/// `DefaultPasscodeKdf` — the cost every new wrap is written with.
///
/// The C++ build picks Argon2id only when OpenSSL exposes the algorithm and
/// falls back to scrypt otherwise; this port always links RustCrypto's Argon2id
/// (v1.3, the same variant OpenSSL provides), so the preferred family is
/// always available.
#[must_use]
pub fn default_passcode_kdf() -> PasscodeKdf {
    PasscodeKdf {
        kind: KDF_ARGON2ID,
        memory: ARGON2_MEMORY_KIB,
        time: ARGON2_TIME,
        parallel: ARGON2_LANES,
    }
}

/// `CreatePasscodeKey` — derives the 256-byte local key from a passcode.
///
/// Returns `None` for an empty passcode, a salt shorter than
/// [`SALT_MIN_SIZE`], or a record that is invalid or over the read ceilings —
/// exactly the cases the C++ helper reports with a null key.
#[must_use]
pub fn create_passcode_key(
    passcode: &[u8],
    salt: &[u8],
    kdf: &PasscodeKdf,
) -> Option<LocalKey> {
    if passcode.is_empty()
        || salt.len() < SALT_MIN_SIZE
        || !kdf.cost_within_limits()
        || !kdf.valid()
    {
        return None;
    }
    let mut key = [0u8; LOCAL_KEY_SIZE];
    if kdf.kind == KDF_ARGON2ID {
        let params = Params::new(kdf.memory, kdf.time, kdf.parallel, Some(LOCAL_KEY_SIZE))
            .ok()?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        argon.hash_password_into(passcode, salt, &mut key).ok()?;
    } else if kdf.kind == KDF_SCRYPT {
        let log_n = u8::try_from(kdf.memory.max(2).trailing_zeros()).ok()?;
        // `Params` only records a key length for the password-hash wrapper and
        // caps it at 64; the derived length is whatever output slice `scrypt()`
        // is handed, and a tdata key is 256 bytes.
        let params = scrypt::Params::new(log_n, kdf.time, kdf.parallel, 64).ok()?;
        scrypt::scrypt(passcode, salt, &params, &mut key).ok()?;
    } else {
        return None;
    }
    Some(LocalKey::from_bytes(key))
}