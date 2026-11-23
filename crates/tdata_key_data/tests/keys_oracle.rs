//! Bit-exact oracle for the key derivations and the key filenames.
//!
//! Vectors come from `tests/oracle/gen_oracle.py`: PBKDF2 through Python's
//! `hashlib`, and the passcode wraps from `openssl kdf` — the library the C++
//! build calls.

mod vectors;

use tdata_key_data::file_key::to_file_part;
use tdata_key_data::local_key::{create_legacy_local_key, create_local_key};
use tdata_key_data::passcode::{
    create_passcode_key, default_passcode_kdf, PasscodeKdf, ARGON2_MEMORY_KIB, ARGON2_TIME,
    KDF_ARGON2ID, KDF_SCRYPT, SCRYPT_N,
};

fn salt_32() -> Vec<u8> {
    hex::decode(vectors::SALT_32_HEX).unwrap()
}

fn salt_16() -> Vec<u8> {
    hex::decode(vectors::SALT_16_HEX).unwrap()
}

#[test]
fn local_key_with_passcode_matches_oracle() {
    let key = create_local_key(vectors::PASSCODE, &salt_32());
    assert_eq!(hex::encode(key.as_bytes()), vectors::LOCAL_KEY_WITH_PASSCODE_HEX);
}

#[test]
fn local_key_without_passcode_matches_oracle() {
    let key = create_local_key(b"", &salt_32());
    assert_eq!(hex::encode(key.as_bytes()), vectors::LOCAL_KEY_NO_PASSCODE_HEX);
}

#[test]
fn legacy_local_key_with_passcode_matches_oracle() {
    let key = create_legacy_local_key(vectors::PASSCODE, &salt_32());
    assert_eq!(
        hex::encode(key.as_bytes()),
        vectors::LEGACY_KEY_WITH_PASSCODE_HEX
    );
}

#[test]
fn legacy_local_key_without_passcode_matches_oracle() {
    let key = create_legacy_local_key(b"", &salt_32());
    assert_eq!(
        hex::encode(key.as_bytes()),
        vectors::LEGACY_KEY_NO_PASSCODE_HEX
    );
}

#[test]
fn default_passcode_kdf_is_argon2id_with_recorded_costs() {
    let kdf = default_passcode_kdf();
    assert_eq!(kdf.kind, KDF_ARGON2ID);
    assert_eq!(kdf.memory, ARGON2_MEMORY_KIB);
    assert_eq!(kdf.time, ARGON2_TIME);
    assert_eq!(kdf.parallel, 1);
    assert!(kdf.valid());
    assert!(kdf.cost_within_limits());
}

#[test]
fn passcode_argon2id_matches_openssl() {
    let key = create_passcode_key(vectors::PASSCODE, &salt_16(), &default_passcode_kdf())
        .expect("default costs are usable");
    assert_eq!(hex::encode(key.as_bytes()), vectors::PASSCODE_ARGON2ID_HEX);
}

#[test]
fn passcode_scrypt_matches_openssl() {
    let kdf = PasscodeKdf {
        kind: KDF_SCRYPT,
        memory: SCRYPT_N,
        time: 8,
        parallel: 1,
    };
    let key = create_passcode_key(vectors::PASSCODE, &salt_16(), &kdf).expect("valid scrypt");
    assert_eq!(hex::encode(key.as_bytes()), vectors::PASSCODE_SCRYPT_HEX);
}

#[test]
fn passcode_scrypt_small_matches_openssl() {
    // N 65536, r 4 — the numbers a 32-bit Windows build records.
    let kdf = PasscodeKdf {
        kind: KDF_SCRYPT,
        memory: 65_536,
        time: 4,
        parallel: 1,
    };
    let key = create_passcode_key(vectors::PASSCODE, &salt_16(), &kdf).expect("valid scrypt");
    assert_eq!(
        hex::encode(key.as_bytes()),
        vectors::PASSCODE_SCRYPT_SMALL_HEX
    );
}

#[test]
fn passcode_rejections_follow_the_cpp_checks() {
    let good = default_passcode_kdf();
    assert!(create_passcode_key(b"", &salt_16(), &good).is_none());
    assert!(create_passcode_key(b"passcode", &[0u8; 7], &good).is_none());
    assert!(create_passcode_key(b"passcode", &salt_16(), &PasscodeKdf::default()).is_none());
    let too_costly = PasscodeKdf {
        memory: 8 * 1024 * 1024,
        ..good
    };
    assert!(!too_costly.cost_within_limits());
    assert!(create_passcode_key(b"passcode", &salt_16(), &too_costly).is_none());
    let unknown = PasscodeKdf {
        kind: 7,
        ..good
    };
    assert!(!unknown.valid());
    assert!(create_passcode_key(b"passcode", &salt_16(), &unknown).is_none());
}

#[test]
fn kdf_validity_rules() {
    let argon = PasscodeKdf {
        kind: KDF_ARGON2ID,
        memory: 8,
        time: 1,
        parallel: 1,
    };
    assert!(argon.valid());
    assert!(!PasscodeKdf { memory: 7, ..argon }.valid());
    assert!(!PasscodeKdf { time: 0, ..argon }.valid());
    assert!(!PasscodeKdf { parallel: 0, ..argon }.valid());

    let scrypt_kdf = PasscodeKdf {
        kind: KDF_SCRYPT,
        memory: 65_536,
        time: 8,
        parallel: 1,
    };
    assert!(scrypt_kdf.valid());
    assert!(!PasscodeKdf { memory: 65_535, ..scrypt_kdf }.valid());
    assert!(!PasscodeKdf { memory: 1, ..scrypt_kdf }.valid());
    assert!(scrypt_kdf.cost_within_limits());
}

#[test]
fn file_part_matches_oracle() {
    assert_eq!(to_file_part(0x0123456789ABCDEF), vectors::FILE_PART_OF_ABCDEF_HEX);
    assert_eq!(to_file_part(1), vectors::FILE_PART_OF_ONE_HEX);
    assert_eq!(to_file_part(u64::MAX), vectors::FILE_PART_OF_MAX_HEX);
    // The low nibble comes first, so this is not the usual hex rendering.
    assert_eq!(to_file_part(0x0F), "F000000000000000");
    assert_eq!(to_file_part(0), "0000000000000000");
}