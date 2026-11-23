//! The `QDataStream` shape of a serialized `DcOptions` blob.
//!
//! Source of truth: `DcOptions::serialize` and
//! `DcOptions::constructFromSerialized` in `mtproto/mtproto_dc_options.cpp`.
//!
//! Layout (all `qint32` big-endian unless noted):
//!
//! ```text
//! -version              negative on purpose: -2 means "format 2"
//! count                 endpoints follow
//! per endpoint:         id, flags, port, ipSize, ip bytes,
//!                       secretSize, secret bytes   (secret only when version > 0)
//! cdnKeyCount           only present when the stream has more and version > 1
//! per key:              dcId, bytes(n), bytes(e)    (bytes = size + raw)
//! ```
//!
//! A version of `0` means the oldest shape: the first field is the endpoint
//! count itself, with no secret section.

/// `MTP::kVersion` of the options blob.
pub const VERSION: i32 = 2;
/// The longest IP address the reader accepts (`kMaxIpSize`).
pub const MAX_IP_SIZE: i32 = 45;
/// The longest proxy secret the reader accepts (`kMaxSecretSize`).
pub const MAX_SECRET_SIZE: i32 = 32;

/// A cursor over one serialized blob, failing stickily like `QDataStream`.
#[derive(Debug, Clone)]
pub struct BlobReader<'a> {
    data: &'a [u8],
    position: usize,
    ok: bool,
}

impl<'a> BlobReader<'a> {
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: 0,
            ok: true,
        }
    }

    #[must_use]
    pub fn at_end(&self) -> bool {
        self.position >= self.data.len()
    }

    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// `stream >> qint32`.
    pub fn read_i32(&mut self) -> Option<i32> {
        if !self.ok {
            return None;
        }
        let end = self.position + 4;
        let bytes = self.data.get(self.position..end)?;
        self.position = end;
        Some(i32::from_be_bytes(bytes.try_into().ok()?))
    }

    /// `stream.readRawData(buffer, size)`.
    pub fn read_raw(&mut self, size: usize) -> Option<&'a [u8]> {
        if !self.ok {
            return None;
        }
        let end = self.position + size;
        let bytes = self.data.get(self.position..end)?;
        self.position = end;
        Some(bytes)
    }
}

/// A writer producing the same shape.
#[derive(Debug, Default, Clone)]
pub struct BlobWriter {
    data: Vec<u8>,
}

impl BlobWriter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[must_use]
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    pub fn write_i32(&mut self, value: i32) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn write_raw(&mut self, bytes: &[u8]) -> &mut Self {
        self.data.extend_from_slice(bytes);
        self
    }

    /// `Serialize::bytes` — a length-prefixed byte vector.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> &mut Self {
        self.write_i32(bytes.len() as i32);
        self.write_raw(bytes)
    }
}