//! `FileKey` filenames: the 16-character hex part and the `0`/`1`/`s` postfixes.
//!
//! Source of truth: `ToFilePart`, `KeyAlreadyUsed`, `GenerateKey` and
//! `ClearKey` in `storage_file_utilities.cpp`, plus `using FileKey = quint64`
//! in `storage/storage_account.h`.
//!
//! A key names one record, and each record may exist up to three times:
//! `0` the legacy plain file, `1` the legacy backup, `s` the current file
//! written through a temporary. The hex part is written *least* significant
//! nibble first, so the mapping is not the usual hex rendering.

use std::fs;
use std::path::{Path, PathBuf};

/// `FileKey` — the 64-bit record name stored in the account map.
pub type FileKey = u64;

/// The characters a record name may carry, in the order they are tried.
pub const POSTFIXES: [char; 3] = ['0', '1', 's'];

/// Appends `text` to a path without introducing a separator.
#[must_use]
pub fn append_to_path(path: &Path, text: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(text);
    PathBuf::from(value)
}

/// `path` followed by one postfix character.
#[must_use]
pub fn with_postfix(path: &Path, postfix: char) -> PathBuf {
    append_to_path(path, &postfix.to_string())
}

/// `ToFilePart` — 16 uppercase characters, low nibble first.
#[must_use]
pub fn to_file_part(val: FileKey) -> String {
    let mut result = String::with_capacity(0x10);
    let mut val = val;
    for _ in 0..0x10 {
        let v = (val & 0x0F) as u8;
        result.push(if v < 0x0A {
            char::from(b'0' + v)
        } else {
            char::from(b'A' + (v - 0x0A))
        });
        val >>= 4;
    }
    result
}

/// `KeyAlreadyUsed` — whether any postfix of `base` exists.
///
/// The C++ helper appends the postfixes to the caller's string; here the base
/// path stays untouched and only the checks are shared.
#[must_use]
pub fn key_already_used(base: &Path) -> bool {
    POSTFIXES
        .iter()
        .any(|postfix| with_postfix(base, *postfix).exists())
}

/// `GenerateKey` — a fresh non-zero key whose file names are all free.
#[must_use]
pub fn generate_key(base_path: &Path) -> FileKey {
    loop {
        let result: FileKey = rand::random();
        if result != 0 && !key_already_used(&base_path.join(to_file_part(result))) {
            return result;
        }
    }
}

/// `ClearKey` — removes every postfix of one record.
pub fn clear_key(key: FileKey, base_path: &Path) {
    let base = base_path.join(to_file_part(key));
    for postfix in POSTFIXES {
        let _ = fs::remove_file(with_postfix(&base, postfix));
    }
}