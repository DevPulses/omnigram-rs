//! Bit-exact oracle for the encrypted block and the container frame.
//!
//! Vectors come from `tests/oracle/gen_oracle.py`, which chains AES-IGE over
//! AES-ECB by hand and hashes with `hashlib` — nothing is shared with the port.

mod vectors;

use sha1::{Digest, Sha1};
use tdata_key_data::container::{header_bytes, read_signed, sign, TDF_MAGIC};
use tdata_key_data::decrypt::decrypt_local;
use tdata_key_data::descriptor::{EncryptedDescriptor, NULL_BYTE_ARRAY_LENGTH};
use tdata_key_data::local_cipher::{aes_encrypt_local, Key128};
use tdata_key_data::local_key::LocalKey;

fn local_key() -> LocalKey {
    LocalKey::from_slice(&hex::decode(vectors::LOCAL_KEY_HEX).unwrap()).unwrap()
}

fn block_one() -> Vec<u8> {
    hex::decode(vectors::BLOCK_ONE_HEX).unwrap()
}

fn block_three() -> Vec<u8> {
    hex::decode(vectors::BLOCK_THREE_HEX).unwrap()
}

/// The payload the generator built: prefix, block, null block, block.
fn payload() -> EncryptedDescriptor {
    let mut data = EncryptedDescriptor::with_reserved_size(60);
    data.write_data(&block_one());
    data.write_null_data();
    data.write_data(&block_three());
    data
}

/// Wraps an arbitrary plaintext block the way `PrepareEncrypted` would, so the
/// rejection paths of `DecryptLocal` can be exercised with a well-signed blob.
fn wrap(plain: &[u8], key: &LocalKey) -> Vec<u8> {
    let mut key128: Key128 = [0u8; 16];
    key128.copy_from_slice(&Sha1::digest(plain)[..16]);
    let encrypted = aes_encrypt_local(plain, key, &key128).unwrap();
    let mut blob = key128.to_vec();
    blob.extend_from_slice(&encrypted);
    blob
}

#[test]
fn payload_layout_matches_oracle() {
    assert_eq!(hex::encode(payload().data()), vectors::PAYLOAD_HEX);
}

#[test]
fn prepared_block_matches_oracle() {
    let key = local_key();
    let mut data = payload();
    let encrypted = tdata_key_data::prepare::prepare_encrypted_with(&mut data, &key, |padding| {
        padding.fill(vectors::PAD_BYTE);
    });
    assert_eq!(hex::encode(&encrypted), vectors::ENCRYPTED_HEX);
}

#[test]
fn decrypted_block_round_trips_to_the_payload() {
    let key = local_key();
    let encrypted = hex::decode(vectors::ENCRYPTED_HEX).unwrap();
    let decrypted = decrypt_local(&encrypted, &key).expect("the oracle block opens");
    // The length prefix holds the payload size after `PrepareEncrypted`, so the
    // decrypted payload differs from the descriptor only in those four bytes.
    let mut expected = hex::decode(vectors::PAYLOAD_HEX).unwrap();
    let size = expected.len() as u32;
    expected[..4].copy_from_slice(&size.to_le_bytes());
    assert_eq!(hex::encode(&decrypted.data), hex::encode(&expected));
    assert_eq!(decrypted.position, 4);

    let mut reader = decrypted.reader();
    assert_eq!(reader.read_byte_array(), Some(Some(&block_one()[..])));
    assert_eq!(reader.read_byte_array(), Some(None));
    assert_eq!(reader.read_byte_array(), Some(Some(&block_three()[..])));
    assert_eq!(reader.read_byte_array(), None);
    assert!(reader.is_ok());
}

#[test]
fn container_frame_matches_oracle() {
    let body = hex::decode(vectors::FILE_BODY_HEX).unwrap();
    let mut expected = header_bytes(vectors::APP_VERSION).to_vec();
    expected.extend_from_slice(&body);
    expected.extend_from_slice(&sign(&body, vectors::APP_VERSION));
    assert_eq!(hex::encode(&expected), vectors::FILE_HEX);
    assert_eq!(&expected[..4], &TDF_MAGIC);
}

#[test]
fn signed_file_parses_back() {
    let file = hex::decode(vectors::FILE_HEX).unwrap();
    let parsed = read_signed(&file, vectors::APP_VERSION).expect("the oracle file verifies");
    assert_eq!(parsed.version, vectors::APP_VERSION);
    assert_eq!(hex::encode(&parsed.body), vectors::FILE_BODY_HEX);

    let mut reader = tdata_key_data::descriptor::DataReader::new(&parsed.body, 0);
    let stored = reader.read_byte_array().flatten().expect("one block");
    assert_eq!(hex::encode(stored), vectors::ENCRYPTED_HEX);
}

#[test]
fn container_rejections_follow_read_file() {
    let file = hex::decode(vectors::FILE_HEX).unwrap();

    let mut bad_magic = file.clone();
    bad_magic[0] ^= 0x01;
    assert!(read_signed(&bad_magic, vectors::APP_VERSION).is_none());

    let mut newer = file.clone();
    newer[4..8].copy_from_slice(&(vectors::APP_VERSION + 1).to_le_bytes());
    assert!(read_signed(&newer, vectors::APP_VERSION).is_none());

    let mut tampered = file.clone();
    let middle = tampered.len() / 2;
    tampered[middle] ^= 0x01;
    assert!(read_signed(&tampered, vectors::APP_VERSION).is_none());

    let truncated = &file[..8];
    assert!(read_signed(truncated, vectors::APP_VERSION).is_none());

    // An older version is accepted: only a newer one is refused.
    let mut older = file.clone();
    older[4..8].copy_from_slice(&(vectors::APP_VERSION - 1).to_le_bytes());
    let resigned = {
        let body = older[8..older.len() - 16].to_vec();
        let mut value = older[..8].to_vec();
        value.extend_from_slice(&body);
        value.extend_from_slice(&sign(&body, vectors::APP_VERSION - 1));
        value
    };
    let parsed = read_signed(&resigned, vectors::APP_VERSION).expect("older version is fine");
    assert_eq!(parsed.version, vectors::APP_VERSION - 1);
}

#[test]
fn decrypt_rejects_damaged_blocks() {
    let key = local_key();
    let encrypted = hex::decode(vectors::ENCRYPTED_HEX).unwrap();

    assert!(decrypt_local(&encrypted[..16], &key).is_none());
    let mut unaligned = encrypted.clone();
    unaligned.push(0);
    assert!(decrypt_local(&unaligned, &key).is_none());

    let mut wrong_header = encrypted.clone();
    wrong_header[0] ^= 0x01;
    assert!(decrypt_local(&wrong_header, &key).is_none());

    let mut wrong_key = hex::decode(vectors::LOCAL_KEY_HEX).unwrap();
    // Byte 8 is the first byte the local derivation actually reads; bytes 0..8
    // are unused by `prepareAES_oldmtp`, so flipping those would change nothing.
    wrong_key[8] ^= 0x01;
    let other = LocalKey::from_slice(&wrong_key).unwrap();
    assert!(decrypt_local(&encrypted, &other).is_none());
}

#[test]
fn decrypt_rejects_bad_size_prefixes() {
    let key = local_key();

    let mut too_big = vec![0u8; 32];
    too_big[..4].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
    assert!(decrypt_local(&wrap(&too_big, &key), &key).is_none());

    let mut too_small = vec![0u8; 32];
    too_small[..4].copy_from_slice(&3u32.to_le_bytes());
    assert!(decrypt_local(&wrap(&too_small, &key), &key).is_none());

    let mut no_padding_room = vec![0u8; 32];
    no_padding_room[..4].copy_from_slice(&4u32.to_le_bytes());
    assert!(decrypt_local(&wrap(&no_padding_room, &key), &key).is_none());

    let mut good = vec![0u8; 32];
    good[..4].copy_from_slice(&32u32.to_le_bytes());
    let decrypted = decrypt_local(&wrap(&good, &key), &key).expect("32 bytes of data fit");
    assert_eq!(decrypted.data.len(), 32);
}

#[test]
fn null_block_is_serialized_like_qdatastream() {
    let mut data = EncryptedDescriptor::default();
    data.write_null_data();
    let expected = NULL_BYTE_ARRAY_LENGTH.to_be_bytes();
    assert_eq!(&data.data()[4..], &expected);
    assert_eq!(data.data().len(), 8);
}