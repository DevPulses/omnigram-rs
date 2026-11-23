//! Reading one container back: file candidates, signature check and decryption.
//!
//! Source of truth: `ReadFile`, `ReadEncryptedFile` (both overloads) in
//! `storage_file_utilities.cpp`.
//!
//! Two files may hold one record: the current one with the `s` postfix, or the
//! legacy `0`/`1` pair. The modern file wins when it exists; otherwise the newer
//! of the legacy pair is tried first and the loser is removed after a successful
//! read, so the record collapses back to a single file.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::container::read_signed;
use crate::decrypt::decrypt_local;
use crate::descriptor::DataReader;
use crate::file_key::{to_file_part, with_postfix, FileKey};
use crate::local_key::LocalKey;

/// A file whose header and signature checked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileReadDescriptor {
    /// The version recorded in the file, already known to be `<= my_version`.
    pub version: i32,
    /// The blocks, positioned at [`FileReadDescriptor::position`].
    pub data: Vec<u8>,
    /// Where the stream starts: 0 for a plain file, 4 for a decrypted payload.
    pub position: usize,
}

impl FileReadDescriptor {
    /// A reader over the blocks, positioned like the C++ stream.
    #[must_use]
    pub fn reader(&self) -> DataReader<'_> {
        DataReader::new(&self.data, self.position)
    }
}

/// `ReadFile` — opens `base_path/name` through its candidate postfixes.
#[must_use]
pub fn read_file(name: &str, base_path: &Path, my_version: i32) -> Option<FileReadDescriptor> {
    let base = base_path.join(name);
    let mut to_try: [Option<PathBuf>; 2] = [None, None];

    let modern = with_postfix(&base, 's');
    if modern.exists() {
        to_try[0] = Some(modern);
    } else {
        let plain = with_postfix(&base, '0');
        if plain.exists() {
            let backup = with_postfix(&base, '1');
            let plain_time = last_modified(&plain);
            let backup_time = last_modified(&backup);
            let (first, second) = if backup.exists() && plain_time < backup_time {
                (backup, plain)
            } else {
                (plain, backup)
            };
            to_try[0] = Some(first);
            if second.exists() {
                to_try[1] = Some(second);
            }
        } else {
            to_try[0] = Some(with_postfix(&base, '1'));
        }
    }

    for i in 0..2 {
        let Some(path) = to_try[i].clone() else {
            break;
        };
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let Some(signed) = read_signed(&bytes, my_version) else {
            continue;
        };
        if i == 1 || (i == 0 && to_try[1].is_some()) {
            if let Some(other) = to_try[1 - i].as_ref() {
                let _ = fs::remove_file(other);
            }
        }
        return Some(FileReadDescriptor {
            version: signed.version,
            data: signed.body,
            position: 0,
        });
    }
    None
}

/// `ReadEncryptedFile` — reads the file, then opens its single encrypted block.
#[must_use]
pub fn read_encrypted_file(
    name: &str,
    base_path: &Path,
    local_key: &LocalKey,
    my_version: i32,
) -> Option<FileReadDescriptor> {
    let mut result = read_file(name, base_path, my_version)?;
    let encrypted = {
        let mut reader = result.reader();
        reader
            .read_byte_array()
            .flatten()
            .map_or_else(Vec::new, <[u8]>::to_vec)
    };
    let decrypted = decrypt_local(&encrypted, local_key)?;
    result.data = decrypted.data;
    result.position = decrypted.position;
    Some(result)
}

/// `ReadEncryptedFile` for a [`FileKey`] record name.
#[must_use]
pub fn read_encrypted_file_for(
    key: FileKey,
    base_path: &Path,
    local_key: &LocalKey,
    my_version: i32,
) -> Option<FileReadDescriptor> {
    read_encrypted_file(&to_file_part(key), base_path, local_key, my_version)
}

/// Modification time used to order the legacy pair; a missing file sorts first.
fn last_modified(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}