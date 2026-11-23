//! Building one container file: blocks, signature and the commit into the queue.
//!
//! Source of truth: `FileWriteDescriptor` in `storage_file_utilities.cpp`.
//!
//! A descriptor buffers the blocks, keeps the MD5 state updated as they are
//! written and, on [`FileWriteDescriptor::finish`], signs the file and hands it
//! to the write manager — synchronously or not, depending on how it was created.
//! `finish` is idempotent: the destructor runs it, a repeated call reports the
//! remembered result and never writes twice.

use std::path::{Path, PathBuf};

use md5::{Digest, Md5};

use crate::container::{sign, SIGNATURE_LEN, TDF_MAGIC};
use crate::descriptor::{write_byte_array, EncryptedDescriptor};
use crate::file_key::{to_file_part, FileKey};
use crate::local_key::LocalKey;
use crate::manager::{self, WriteEntry};
use crate::prepare::prepare_encrypted;
use crate::version::APP_VERSION;

/// `FileWriteDescriptor` — one file being assembled and committed.
#[derive(Debug)]
pub struct FileWriteDescriptor {
    base_path: PathBuf,
    base: PathBuf,
    safe_data: Vec<u8>,
    md5: Md5,
    full_size: i32,
    sync: bool,
    result: bool,
    finished: bool,
}

impl FileWriteDescriptor {
    /// A descriptor for a named file, e.g. `map` or `config`.
    #[must_use]
    pub fn new(name: &str, base_path: &Path, sync: bool) -> Self {
        let base = base_path.join(name);
        Self {
            base_path: base_path.to_path_buf(),
            base,
            safe_data: Vec::new(),
            md5: Md5::new(),
            full_size: 0,
            sync,
            result: true,
            finished: false,
        }
    }

    /// A descriptor for a [`FileKey`] record.
    #[must_use]
    pub fn for_key(key: FileKey, base_path: &Path, sync: bool) -> Self {
        Self::new(&to_file_part(key), base_path, sync)
    }

    /// `writeData` — appends one block and feeds it into the signature.
    pub fn write_data(&mut self, data: Option<&[u8]>) {
        if self.finished {
            return;
        }
        write_byte_array(&mut self.safe_data, data);
        let (length, bytes) = match data {
            Some(bytes) => (bytes.len() as u32, bytes),
            None => (crate::descriptor::NULL_BYTE_ARRAY_LENGTH, &[][..]),
        };
        // `HashMd5::feed` sees the length big-endian, matching `qbswap`.
        self.md5.update(length.to_be_bytes());
        self.md5.update(bytes);
        self.full_size += 4 + bytes.len() as i32;
    }

    /// `writeEncrypted` — encrypts the payload and stores it as one block.
    pub fn write_encrypted(&mut self, data: &mut EncryptedDescriptor, key: &LocalKey) {
        let encrypted = prepare_encrypted(data, key);
        self.write_data(Some(&encrypted));
    }

    /// `writeEncrypted` with caller-provided padding, so a whole file can be
    /// reproduced byte for byte against the oracle.
    pub fn write_encrypted_with(
        &mut self,
        data: &mut EncryptedDescriptor,
        key: &LocalKey,
        fill: impl FnMut(&mut [u8]),
    ) {
        let encrypted = crate::prepare::prepare_encrypted_with(data, key, fill);
        self.write_data(Some(&encrypted));
    }

    /// `finish` — signs and commits the file, reporting durable success.
    ///
    /// A synchronous descriptor reports whether the record reached the disk; an
    /// asynchronous one is fire-and-forget and reports `true`.
    pub fn finish(&mut self) -> bool {
        if self.finished {
            return self.result;
        }
        self.finished = true;

        let mut md5 = self.md5.clone();
        md5.update(self.full_size.to_le_bytes());
        md5.update(APP_VERSION.to_le_bytes());
        md5.update(TDF_MAGIC);
        let signature: [u8; SIGNATURE_LEN] = md5.finalize().into();

        let entry = WriteEntry {
            base_path: self.base_path.clone(),
            base: self.base.clone(),
            data: std::mem::take(&mut self.safe_data),
            md5: signature,
        };
        self.result = if self.sync {
            manager::write_sync(entry)
        } else {
            manager::schedule_write(entry);
            true
        };
        self.result
    }

    /// The path this descriptor writes, without the postfix.
    #[must_use]
    pub fn base(&self) -> &Path {
        &self.base
    }

    /// The signature of the blocks as written so far — exposed for tests that
    /// compare a built file against an oracle without touching the disk.
    #[must_use]
    pub fn signature(&self) -> [u8; SIGNATURE_LEN] {
        let mut md5 = self.md5.clone();
        md5.update(self.full_size.to_le_bytes());
        md5.update(APP_VERSION.to_le_bytes());
        md5.update(TDF_MAGIC);
        md5.finalize().into()
    }

    /// The serialized blocks as written so far.
    #[must_use]
    pub fn blocks(&self) -> &[u8] {
        &self.safe_data
    }

    /// The whole file this descriptor would produce: header, blocks, signature.
    ///
    /// Equivalent to [`crate::container::sign`] applied the way `finish` does.
    #[must_use]
    pub fn file_bytes(&self) -> Vec<u8> {
        let mut file = crate::container::header_bytes(APP_VERSION).to_vec();
        file.extend_from_slice(&self.safe_data);
        file.extend_from_slice(&sign(&self.safe_data, APP_VERSION));
        file
    }
}

impl Drop for FileWriteDescriptor {
    fn drop(&mut self) {
        self.finish();
    }
}