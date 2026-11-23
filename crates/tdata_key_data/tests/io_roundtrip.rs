//! Reading, writing and queueing real files: the whole path over a temp folder.
//!
//! These tests pin the file-level behaviour of `ReadFile`, `ReadEncryptedFile`
//! and `WriteManager`: which of the `0`/`1`/`s` candidates wins, what the loser
//! loses, and how the queue coalesces writes around a removal.

mod vectors;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use tdata_key_data::container::{header_bytes, sign};
use tdata_key_data::descriptor::EncryptedDescriptor;
use tdata_key_data::file_key::{
    clear_key, generate_key, key_already_used, to_file_part, with_postfix, FileKey,
};
use tdata_key_data::local_key::LocalKey;
use tdata_key_data::manager::{RemoveEntry, WriteEntry, WriteManager};
use tdata_key_data::reader::{read_encrypted_file, read_encrypted_file_for, read_file};
use tdata_key_data::writer::FileWriteDescriptor;
use tdata_key_data::version::APP_VERSION;

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// A unique directory under the process temp root, removed on drop.
struct TempDir(PathBuf);

impl TempDir {
    fn new(name: &str) -> Self {
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "tdata_key_data_{name}_{}_{unique}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn local_key() -> LocalKey {
    LocalKey::from_slice(&hex::decode(vectors::LOCAL_KEY_HEX).unwrap()).unwrap()
}

fn payload() -> EncryptedDescriptor {
    let mut data = EncryptedDescriptor::with_reserved_size(60);
    data.write_data(&hex::decode(vectors::BLOCK_ONE_HEX).unwrap());
    data.write_null_data();
    data.write_data(&hex::decode(vectors::BLOCK_THREE_HEX).unwrap());
    data
}

/// A descriptor writing the oracle payload with the oracle padding.
fn oracle_descriptor(name: &str, base: &Path, sync: bool) -> FileWriteDescriptor {
    let key = local_key();
    let mut file = FileWriteDescriptor::new(name, base, sync);
    let mut data = payload();
    file.write_encrypted_with(&mut data, &key, |padding| padding.fill(vectors::PAD_BYTE));
    file
}

/// Writes one plain (unencrypted) container holding a single raw block.
fn write_plain_container(path: &Path, block: &[u8]) {
    let mut body = Vec::new();
    body.extend_from_slice(&(block.len() as u32).to_be_bytes());
    body.extend_from_slice(block);
    let mut file = header_bytes(APP_VERSION).to_vec();
    file.extend_from_slice(&body);
    file.extend_from_slice(&sign(&body, APP_VERSION));
    fs::write(path, file).expect("write plain container");
}

#[test]
fn sync_write_produces_the_oracle_file() {
    let dir = TempDir::new("sync");
    let mut file = oracle_descriptor("fixture", dir.path(), true);
    assert!(file.finish(), "a synchronous write reports durable success");

    let safe = with_postfix(&dir.path().join("fixture"), 's');
    assert_eq!(hex::encode(fs::read(&safe).unwrap()), vectors::FILE_HEX);
    assert!(!with_postfix(&dir.path().join("fixture"), '0').exists());
    assert!(!with_postfix(&dir.path().join("fixture"), '1').exists());
    assert!(!dir.path().join("fixtures.tmp").exists());
}

#[test]
fn finish_is_idempotent() {
    let dir = TempDir::new("idempotent");
    let mut file = oracle_descriptor("fixture", dir.path(), true);
    assert!(file.finish());
    let safe = with_postfix(&dir.path().join("fixture"), 's');
    let first = fs::read(&safe).unwrap();
    assert!(file.finish());
    assert_eq!(fs::read(&safe).unwrap(), first);
}

#[test]
fn encrypted_file_reads_back() {
    let dir = TempDir::new("read");
    assert!(oracle_descriptor("fixture", dir.path(), true).finish());

    let key = local_key();
    let read = read_encrypted_file("fixture", dir.path(), &key, APP_VERSION)
        .expect("the file opens with its own key");
    assert_eq!(read.version, APP_VERSION);
    assert_eq!(read.position, 4);
    let mut expected = hex::decode(vectors::PAYLOAD_HEX).unwrap();
    let size = expected.len() as u32;
    expected[..4].copy_from_slice(&size.to_le_bytes());
    assert_eq!(hex::encode(&read.data), hex::encode(&expected));

    let mut reader = read.reader();
    assert_eq!(
        reader.read_byte_array().flatten().unwrap(),
        &hex::decode(vectors::BLOCK_ONE_HEX).unwrap()[..]
    );
}

#[test]
fn wrong_key_opens_nothing() {
    let dir = TempDir::new("wrongkey");
    assert!(oracle_descriptor("fixture", dir.path(), true).finish());

    let mut bytes = hex::decode(vectors::LOCAL_KEY_HEX).unwrap();
    // Bytes 0..8 are not read by the local key derivation, byte 8 is.
    bytes[8] ^= 0x01;
    let other = LocalKey::from_slice(&bytes).unwrap();
    assert!(read_encrypted_file("fixture", dir.path(), &other, APP_VERSION).is_none());
}

#[test]
fn plain_file_reads_its_blocks() {
    let dir = TempDir::new("plain");
    assert!(oracle_descriptor("fixture", dir.path(), true).finish());

    let read = read_file("fixture", dir.path(), APP_VERSION).expect("the file opens");
    let mut reader = read.reader();
    let stored = reader.read_byte_array().flatten().unwrap();
    assert_eq!(hex::encode(stored), vectors::ENCRYPTED_HEX);
}

#[test]
fn newer_legacy_file_wins_and_the_other_is_removed() {
    let dir = TempDir::new("legacy");
    let base = dir.path().join("record");
    let plain = with_postfix(&base, '0');
    let backup = with_postfix(&base, '1');

    write_plain_container(&backup, b"older");
    std::thread::sleep(Duration::from_millis(20));
    write_plain_container(&plain, b"newer");

    let read = read_file("record", dir.path(), APP_VERSION).expect("the newer file opens");
    let mut reader = read.reader();
    assert_eq!(reader.read_byte_array().flatten().unwrap(), b"newer");
    assert!(!backup.exists(), "the losing side of the pair is removed");
    assert!(plain.exists());

    let dir = TempDir::new("legacy2");
    let base = dir.path().join("record");
    let plain = with_postfix(&base, '0');
    let backup = with_postfix(&base, '1');
    write_plain_container(&plain, b"older");
    std::thread::sleep(Duration::from_millis(20));
    write_plain_container(&backup, b"newer");

    let read = read_file("record", dir.path(), APP_VERSION).expect("the newer backup opens");
    let mut reader = read.reader();
    assert_eq!(reader.read_byte_array().flatten().unwrap(), b"newer");
    assert!(!plain.exists(), "the losing side of the pair is removed");
}

#[test]
fn modern_file_wins_over_the_legacy_pair() {
    let dir = TempDir::new("modern");
    let base = dir.path().join("record");
    // The writer itself clears the legacy pair, so the legacy file is created
    // afterwards to leave both candidates on disk.
    assert!(oracle_descriptor("record", dir.path(), true).finish());
    write_plain_container(&with_postfix(&base, '0'), b"legacy");

    let read = read_file("record", dir.path(), APP_VERSION).expect("the modern file opens");
    let mut reader = read.reader();
    assert_eq!(
        hex::encode(reader.read_byte_array().flatten().unwrap()),
        vectors::ENCRYPTED_HEX
    );
    assert!(
        with_postfix(&base, '0').exists(),
        "nothing is removed while the modern file is the only candidate"
    );
}

#[test]
fn broken_modern_file_hides_the_legacy_one() {
    let dir = TempDir::new("broken");
    let base = dir.path().join("record");
    write_plain_container(&with_postfix(&base, '0'), b"legacy");
    assert!(oracle_descriptor("record", dir.path(), true).finish());

    let safe = with_postfix(&base, 's');
    let mut bytes = fs::read(&safe).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    fs::write(&safe, bytes).unwrap();

    assert!(
        read_file("record", dir.path(), APP_VERSION).is_none(),
        "a present-but-broken modern file does not fall back to legacy"
    );
}

#[test]
fn file_keys_name_free_records() {
    let dir = TempDir::new("keys");
    let key: FileKey = generate_key(dir.path());
    assert_ne!(key, 0);

    let base = dir.path().join(to_file_part(key));
    assert!(!key_already_used(&base));

    write_plain_container(&with_postfix(&base, '0'), b"taken");
    assert!(key_already_used(&base));

    clear_key(key, dir.path());
    assert!(!key_already_used(&base));
    assert!(!with_postfix(&base, '0').exists());
}

#[test]
fn encrypted_file_by_key_uses_the_file_part() {
    let dir = TempDir::new("bykey");
    let key: FileKey = 0x0123_4567_89AB_CDEF;
    assert!(oracle_descriptor(&to_file_part(key), dir.path(), true).finish());

    let read = read_encrypted_file_for(key, dir.path(), &local_key(), APP_VERSION)
        .expect("the record opens by key");
    let mut expected = hex::decode(vectors::PAYLOAD_HEX).unwrap();
    let size = expected.len() as u32;
    expected[..4].copy_from_slice(&size.to_le_bytes());
    assert_eq!(hex::encode(&read.data), hex::encode(&expected));
}

fn queue_entry(base: &Path, block: &[u8]) -> WriteEntry {
    let mut body = Vec::new();
    body.extend_from_slice(&(block.len() as u32).to_be_bytes());
    body.extend_from_slice(block);
    WriteEntry {
        base_path: base.parent().unwrap().to_path_buf(),
        base: base.to_path_buf(),
        md5: sign(&body, APP_VERSION),
        data: body,
    }
}

#[test]
fn queued_writes_coalesce_per_record() {
    let dir = TempDir::new("coalesce");
    let base = dir.path().join("record");
    let mut manager = WriteManager::new();
    manager.write(queue_entry(&base, b"first"));
    manager.write(queue_entry(&base, b"second"));
    manager.write_sync_all();

    let safe = with_postfix(&base, 's');
    let read = read_file("record", dir.path(), APP_VERSION).expect("the last write is on disk");
    let mut reader = read.reader();
    assert_eq!(reader.read_byte_array().flatten().unwrap(), b"second");
    assert!(safe.exists());
    assert!(manager.is_empty());
}

#[test]
fn a_removal_splits_the_coalescing_window() {
    let dir = TempDir::new("removal");
    let base = dir.path().join("record");
    let mut manager = WriteManager::new();
    manager.write(queue_entry(&base, b"before"));
    manager.remove(RemoveEntry {
        files: vec![with_postfix(&base, 's')],
        folder: None,
    });
    manager.write(queue_entry(&base, b"after"));
    manager.write_sync_all();

    let read = read_file("record", dir.path(), APP_VERSION).expect("the later write survives");
    let mut reader = read.reader();
    assert_eq!(reader.read_byte_array().flatten().unwrap(), b"after");
}

#[test]
fn write_sync_flushes_pending_operations_first() {
    let dir = TempDir::new("flushfirst");
    let first = dir.path().join("first");
    let second = dir.path().join("second");
    let mut manager = WriteManager::new();
    manager.write(queue_entry(&first, b"one"));
    assert!(manager.write_sync(queue_entry(&second, b"two")));

    for name in ["first", "second"] {
        assert!(with_postfix(&dir.path().join(name), 's').exists());
    }
}

#[test]
fn async_manager_writes_what_it_is_given() {
    let dir = TempDir::new("async");
    let base = dir.path().join("record");
    tdata_key_data::manager::schedule_write(queue_entry(&base, b"async"));
    tdata_key_data::manager::sync();

    let read = read_file("record", dir.path(), APP_VERSION).expect("the write landed");
    let mut reader = read.reader();
    assert_eq!(reader.read_byte_array().flatten().unwrap(), b"async");
}

#[test]
fn removing_a_folder_is_queued_as_well() {
    let dir = TempDir::new("folder");
    let doomed = dir.path().join("doomed");
    fs::create_dir_all(&doomed).unwrap();
    fs::write(doomed.join("file"), b"x").unwrap();
    tdata_key_data::manager::schedule_remove_folder(doomed.clone());
    tdata_key_data::manager::sync();
    assert!(!doomed.exists());
}