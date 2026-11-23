//! The `TDF$` container: header, MD5 file signature and its check.
//!
//! Source of truth: `writeHeader` and the signature part of `ReadFile` in
//! `storage_file_utilities.cpp`.
//!
//! Layout of one file:
//!
//! ```text
//! 'T' 'D' 'F' '$'            magic
//! int32 LE                   AppVersion
//! block*                     uint32 BE length + raw bytes per block
//! byte[16]                   MD5
//! ```
//!
//! The signature covers the blocks *and* the header fields, but in a different
//! order than the file holds them: the block bytes, then `bodySize` (int32 LE),
//! then the version (int32 LE), then the magic.

use md5::{Digest, Md5};

/// `TdfMagic` — the four bytes every tdata file starts with.
pub const TDF_MAGIC: [u8; 4] = *b"TDF$";

/// Length of the magic, and of the header it opens.
pub const MAGIC_LEN: usize = TDF_MAGIC.len();

/// Length of the trailing signature.
pub const SIGNATURE_LEN: usize = 16;

/// The header every file starts with: magic followed by the app version.
#[must_use]
pub fn header_bytes(version: i32) -> [u8; 8] {
    let mut header = [0u8; 8];
    header[..MAGIC_LEN].copy_from_slice(&TDF_MAGIC);
    header[MAGIC_LEN..].copy_from_slice(&version.to_le_bytes());
    header
}

/// `HashMd5` over a file body — the trailing signature.
///
/// `body` is the serialized blocks including their length prefixes.
#[must_use]
pub fn sign(body: &[u8], version: i32) -> [u8; SIGNATURE_LEN] {
    let mut md5 = Md5::new();
    md5.update(body);
    md5.update((body.len() as i32).to_le_bytes());
    md5.update(version.to_le_bytes());
    md5.update(TDF_MAGIC);
    md5.finalize().into()
}

/// A body whose header and signature checked out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedBody {
    /// The version recorded in the file header.
    pub version: i32,
    /// The blocks, with the magic, version and signature stripped.
    pub body: Vec<u8>,
}

/// Validates magic, version and signature of a whole file, stripping the frame.
///
/// Mirrors the three rejections of `ReadFile`: a short header, a version newer
/// than `my_version`, and a mismatched signature — each of which makes the
/// caller fall back to the next candidate file.
#[must_use]
pub fn read_signed(bytes: &[u8], my_version: i32) -> Option<SignedBody> {
    if bytes.len() < MAGIC_LEN + 4 + SIGNATURE_LEN {
        return None;
    } else if bytes[..MAGIC_LEN] != TDF_MAGIC {
        return None;
    }
    let version = i32::from_le_bytes(bytes[MAGIC_LEN..MAGIC_LEN + 4].try_into().ok()?);
    if version > my_version {
        return None;
    }
    let body_end = bytes.len() - SIGNATURE_LEN;
    let body = &bytes[MAGIC_LEN + 4..body_end];
    if sign(body, version) != bytes[body_end..] {
        return None;
    }
    Some(SignedBody {
        version,
        body: body.to_vec(),
    })
}