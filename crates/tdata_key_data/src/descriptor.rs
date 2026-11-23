//! The `[uint32 size][blocks]` payload that gets encrypted, and a matching
//! reader for the blocks of a decrypted payload.
//!
//! Source of truth: `EncryptedDescriptor` and the `QDataStream` usage in
//! `storage_file_utilities.cpp`.
//!
//! The buffer starts with a four-byte placeholder: `PrepareEncrypted` writes
//! the total payload length there just before encrypting, so a block payload
//! is always `4 + blocks` bytes. Blocks themselves are `QDataStream`-framed
//! byte arrays: a big-endian `uint32` length followed by the raw bytes, with
//! `0xFFFFFFFF` standing for a null array (see [`DataReader::read_byte_array`]).

/// `sizeof(uint32)` — the payload length prefix, and the stream position of the
/// first block in both directions.
pub const SIZE_PREFIX: usize = 4;

/// A null `QByteArray` as serialized by `QDataStream`.
pub const NULL_BYTE_ARRAY_LENGTH: u32 = 0xFFFF_FFFF;

/// Appends a `QDataStream`-framed byte array; `None` is a null array.
pub fn write_byte_array(out: &mut Vec<u8>, data: Option<&[u8]>) {
    let length = match data {
        Some(data) => data.len() as u32,
        None => NULL_BYTE_ARRAY_LENGTH,
    };
    out.extend_from_slice(&length.to_be_bytes());
    if let Some(data) = data {
        out.extend_from_slice(data);
    }
}

/// `EncryptedDescriptor` — a write-side payload buffer being filled with blocks.
///
/// Always starts with the four-byte length prefix, which `PrepareEncrypted`
/// overwrites with the payload size. (The C++ default constructor has no stream
/// and silently drops writes; it is only ever used as an empty output sink.)
#[derive(Debug, Clone)]
pub struct EncryptedDescriptor {
    data: Vec<u8>,
}

impl Default for EncryptedDescriptor {
    fn default() -> Self {
        Self::with_reserved_size(0)
    }
}

impl EncryptedDescriptor {
    /// `EncryptedDescriptor(uint32 size)` — reserves the padded payload size
    /// the caller expects to write, so a well-sized writer never reallocates.
    ///
    /// The rounded-up total is only a reservation: the real size is whatever
    /// the blocks come to, exactly like the C++ buffer that grows on demand.
    #[must_use]
    pub fn with_reserved_size(size: u32) -> Self {
        let mut full_size = SIZE_PREFIX + size as usize;
        if full_size & 0x0F != 0 {
            full_size += 0x10 - (full_size & 0x0F);
        }
        let mut data = Vec::with_capacity(full_size);
        data.resize(SIZE_PREFIX, 0);
        Self { data }
    }

    /// `stream << data` — appends one block.
    pub fn write_data(&mut self, data: &[u8]) {
        write_byte_array(&mut self.data, Some(data));
    }

    /// `stream << QByteArray()` — appends a null block.
    pub fn write_null_data(&mut self) {
        write_byte_array(&mut self.data, None);
    }

    /// The payload as prepared so far: `[placeholder][blocks]`.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// The payload for in-place preparation (the length prefix is overwritten).
    pub fn data_mut(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    /// Consumes the descriptor, yielding the unprepared payload.
    #[must_use]
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
}

/// A cursor over the blocks of a payload, mirroring `QDataStream`'s framing and
/// its sticky error state (`CheckStreamStatus`).
#[derive(Debug, Clone)]
pub struct DataReader<'a> {
    data: &'a [u8],
    position: usize,
    ok: bool,
}

impl<'a> DataReader<'a> {
    /// A reader starting at `position` — 0 for a file body, [`SIZE_PREFIX`] for
    /// a decrypted payload whose length prefix is skipped.
    #[must_use]
    pub fn new(data: &'a [u8], position: usize) -> Self {
        Self {
            data,
            position,
            ok: position <= data.len(),
        }
    }

    /// Current position inside the payload.
    #[must_use]
    pub fn position(&self) -> usize {
        self.position
    }

    /// Whether every read so far succeeded.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// Moves the cursor to `position`, failing if it is out of bounds.
    pub fn seek(&mut self, position: usize) -> bool {
        if position <= self.data.len() {
            self.position = position;
            true
        } else {
            self.ok = false;
            false
        }
    }

    /// Reads `length` raw bytes.
    pub fn read_bytes(&mut self, length: usize) -> Option<&'a [u8]> {
        let end = self.position.checked_add(length)?;
        let bytes = self.data.get(self.position..end)?;
        self.position = end;
        Some(bytes)
    }

    /// Reads a big-endian `uint32`.
    pub fn read_u32(&mut self) -> Option<u32> {
        let bytes: [u8; 4] = self.read_bytes(4)?.try_into().ok()?;
        Some(u32::from_be_bytes(bytes))
    }

    /// Reads a `QDataStream` byte array: `None` for a null array, `Some(bytes)`
    /// otherwise, and `None` from the outer `Option` when the stream is short.
    pub fn read_byte_array(&mut self) -> Option<Option<&'a [u8]>> {
        let length = self.read_u32()?;
        if length == NULL_BYTE_ARRAY_LENGTH {
            return Some(None);
        }
        Some(self.read_bytes(length as usize))
    }
}