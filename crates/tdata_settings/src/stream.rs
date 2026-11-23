//! The `QDataStream` wire format the settings blocks are written in.
//!
//! Every settings payload is a `QDataStream` at version `Qt_5_1` with the
//! default big-endian byte order. The primitives here cover what the block
//! scheme uses:
//!
//! | Qt type | wire form |
//! |---|---|
//! | `qint32` / `quint32` | 4 bytes, big-endian |
//! | `qint64` / `quint64` | 8 bytes, big-endian |
//! | `bool` | 1 byte, `0` or `1` |
//! | `QByteArray` | `uint32` length, then the bytes; `0xFFFFFFFF` = null |
//! | `QString` | `uint32` byte length, then UTF-16BE code units; `0xFFFFFFFF` = null |
//! | `QVector<T>` | `uint32` count, then the items |
//! | `QMap<K, V>` | `uint32` count, then key/value pairs in key order |
//!
//! Reads are sticky like `QDataStream::status()`: once a read runs past the
//! end, [`StreamReader::is_ok`] stays `false` and the caller fails the whole
//! block, exactly as `CheckStreamStatus` does.

/// A null `QByteArray` or `QString` length.
pub const NULL_LENGTH: u32 = 0xFFFF_FFFF;

/// A cursor over one block payload.
#[derive(Debug, Clone)]
pub struct StreamReader<'a> {
    data: &'a [u8],
    position: usize,
    ok: bool,
}

impl<'a> StreamReader<'a> {
    /// A reader over a whole payload.
    #[must_use]
    pub fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            position: 0,
            ok: true,
        }
    }

    /// Bytes left in the payload.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.position)
    }

    /// Whether the stream reached its end (`QDataStream::atEnd`).
    #[must_use]
    pub fn at_end(&self) -> bool {
        self.position >= self.data.len()
    }

    /// Whether every read so far succeeded.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// Current offset, for diagnostics.
    #[must_use]
    pub fn position(&self) -> usize {
        self.position
    }

    fn take(&mut self, length: usize) -> Option<&'a [u8]> {
        if !self.ok {
            return None;
        }
        let end = self.position.checked_add(length)?;
        let bytes = self.data.get(self.position..end)?;
        self.position = end;
        Some(bytes)
    }

    fn take_array<const N: usize>(&mut self) -> Option<[u8; N]> {
        let bytes = self.take(N)?;
        bytes.try_into().ok()
    }

    /// `stream >> quint32`.
    pub fn read_u32(&mut self) -> Option<u32> {
        self.take_array::<4>().map(u32::from_be_bytes)
    }

    /// `stream >> qint32`.
    pub fn read_i32(&mut self) -> Option<i32> {
        self.take_array::<4>().map(i32::from_be_bytes)
    }

    /// `stream >> quint64`.
    pub fn read_u64(&mut self) -> Option<u64> {
        self.take_array::<8>().map(u64::from_be_bytes)
    }

    /// `stream >> qint64`.
    pub fn read_i64(&mut self) -> Option<i64> {
        self.take_array::<8>().map(i64::from_be_bytes)
    }

    /// `stream.readRawData(&result, N)` — an opaque fixed-size value such as
    /// `MTP::AuthKey::Data`.
    pub fn read_raw<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take_array::<N>()
    }

    /// `stream >> bool` — one byte since Qt 5.0.
    pub fn read_bool(&mut self) -> Option<bool> {
        self.take(1).map(|byte| byte[0] != 0)
    }

    /// `stream >> QByteArray` — a null array reads back as empty.
    pub fn read_byte_array(&mut self) -> Option<Vec<u8>> {
        let length = self.read_u32()?;
        if length == NULL_LENGTH {
            return Some(Vec::new());
        }
        self.take(length as usize).map(<[u8]>::to_vec)
    }

    /// `stream >> QString` — a null string reads back as empty.
    pub fn read_string(&mut self) -> Option<String> {
        let length = self.read_u32()?;
        if length == NULL_LENGTH {
            return Some(String::new());
        }
        let bytes = self.take(length as usize)?;
        let units = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect::<Vec<u16>>();
        Some(String::from_utf16_lossy(&units))
    }

    /// `stream >> QVector<QPair<quint64, ushort>>` — the recent-emoji and
    /// recent-sticker preload shape.
    pub fn read_pairs_u64_u16(&mut self) -> Option<Vec<(u64, u16)>> {
        let count = self.read_u32()?;
        let mut result = Vec::with_capacity(count.min(1 << 16) as usize);
        for _ in 0..count {
            let key = self.read_u64()?;
            let value = self
                .take_array::<2>()
                .map(u16::from_be_bytes)?;
            result.push((key, value));
        }
        Some(result)
    }

    /// `stream >> QVector<QPair<quint32, ushort>>` — the oldest recent-emoji
    /// shape.
    pub fn read_pairs_u32_u16(&mut self) -> Option<Vec<(u32, u16)>> {
        let count = self.read_u32()?;
        let mut result = Vec::with_capacity(count.min(1 << 16) as usize);
        for _ in 0..count {
            let key = self.read_u32()?;
            let value = self.take_array::<2>().map(u16::from_be_bytes)?;
            result.push((key, value));
        }
        Some(result)
    }

    /// `stream >> QVector<QPair<QString, ushort>>` — recent emoji already
    /// stored as string ids.
    pub fn read_pairs_string_u16(&mut self) -> Option<Vec<(String, u16)>> {
        let count = self.read_u32()?;
        let mut result = Vec::with_capacity(count.min(1 << 16) as usize);
        for _ in 0..count {
            let key = self.read_string()?;
            let value = self.take_array::<2>().map(u16::from_be_bytes)?;
            result.push((key, value));
        }
        Some(result)
    }

    /// `stream >> QMap<quint32, quint64>`.
    pub fn read_map_u32_u64(&mut self) -> Option<Vec<(u32, u64)>> {
        let count = self.read_u32()?;
        let mut result = Vec::with_capacity(count.min(1 << 16) as usize);
        for _ in 0..count {
            let key = self.read_u32()?;
            let value = self.read_u64()?;
            result.push((key, value));
        }
        Some(result)
    }

    /// `stream >> QMap<quint64, qint32>`.
    pub fn read_map_u64_i32(&mut self) -> Option<Vec<(u64, i32)>> {
        let count = self.read_u32()?;
        let mut result = Vec::with_capacity(count.min(1 << 16) as usize);
        for _ in 0..count {
            let key = self.read_u64()?;
            let value = self.read_i32()?;
            result.push((key, value));
        }
        Some(result)
    }

    /// `stream >> QMap<QString, int>`.
    pub fn read_map_string_i32(&mut self) -> Option<Vec<(String, i32)>> {
        let count = self.read_u32()?;
        let mut result = Vec::with_capacity(count.min(1 << 16) as usize);
        for _ in 0..count {
            let key = self.read_string()?;
            let value = self.read_i32()?;
            result.push((key, value));
        }
        Some(result)
    }
}

/// A writer producing the same wire format, used by the write path and by the
/// tests that rebuild an oracle payload byte for byte.
#[derive(Debug, Default, Clone)]
pub struct StreamWriter {
    data: Vec<u8>,
}

impl StreamWriter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The bytes written so far.
    #[must_use]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// Consumes the writer, yielding its bytes.
    #[must_use]
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }

    pub fn write_u32(&mut self, value: u32) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn write_u16(&mut self, value: u16) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn write_i32(&mut self, value: i32) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn write_u64(&mut self, value: u64) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn write_i64(&mut self, value: i64) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    pub fn write_bool(&mut self, value: bool) -> &mut Self {
        self.data.push(u8::from(value));
        self
    }

    pub fn write_byte_array(&mut self, value: &[u8]) -> &mut Self {
        self.write_u32(value.len() as u32);
        self.data.extend_from_slice(value);
        self
    }

    pub fn write_string(&mut self, value: &str) -> &mut Self {
        let units = value.encode_utf16().collect::<Vec<u16>>();
        self.write_u32((units.len() * 2) as u32);
        for unit in units {
            self.data.extend_from_slice(&unit.to_be_bytes());
        }
        self
    }

    pub fn write_pairs_string_u16(&mut self, value: &[(String, u16)]) -> &mut Self {
        self.write_u32(value.len() as u32);
        for (key, item) in value {
            self.write_string(key);
            self.data.extend_from_slice(&item.to_be_bytes());
        }
        self
    }

    pub fn write_pairs_u64_u16(&mut self, value: &[(u64, u16)]) -> &mut Self {
        self.write_u32(value.len() as u32);
        for (key, item) in value {
            self.write_u64(*key);
            self.data.extend_from_slice(&item.to_be_bytes());
        }
        self
    }
}