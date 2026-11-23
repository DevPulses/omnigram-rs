//! `ByteArrayReader` — reads one storage buffer back.
//!
//! Source of truth: `storage/serialize_common.{h,cpp}`.
//!
//! Failure is sticky and *quiet*: the C++ `operator>>` wrapper assigns the
//! value's default whenever the stream is or becomes bad, so a reader never
//! throws and never leaves a half-filled field. Here every read returns
//! `None` in exactly those cases, and [`ByteArrayReader::is_ok`] stays `false`
//! for the rest of the buffer; callers treat `None` as that default, which is
//! what `unwrap_or_default()` gives them.

use crate::color::Color;
use crate::date_time::SerializedDateTime;
use crate::sizes::LENGTH_SIZE;
use crate::writer::NULL_LENGTH;

/// `Serialize::ByteArrayReader`.
#[derive(Debug, Clone)]
pub struct ByteArrayReader {
    data: Vec<u8>,
    position: usize,
    ok: bool,
}

impl ByteArrayReader {
    /// `ByteArrayReader(QByteArray)`.
    #[must_use]
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data,
            position: 0,
            ok: true,
        }
    }

    /// A reader over a borrowed buffer.
    #[must_use]
    pub fn from_slice(data: &[u8]) -> Self {
        Self::new(data.to_vec())
    }

    /// `atEnd()`.
    #[must_use]
    pub fn at_end(&self) -> bool {
        self.position >= self.data.len()
    }

    /// `ok()` — the stream is still readable.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        self.ok
    }

    /// The offset the next read starts at.
    #[must_use]
    pub fn position(&self) -> usize {
        self.position
    }

    fn fail(&mut self) {
        self.ok = false;
    }

    fn take(&mut self, length: usize) -> Option<&[u8]> {
        if !self.ok {
            return None;
        }
        let Some(end) = self.position.checked_add(length) else {
            self.fail();
            return None;
        };
        if end > self.data.len() {
            self.fail();
            return None;
        }
        let start = self.position;
        self.position = end;
        Some(&self.data[start..end])
    }

    /// `stream >> quint32`.
    pub fn read_u32(&mut self) -> Option<u32> {
        self.take(4)
            .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
            .map(u32::from_be_bytes)
    }

    /// `stream >> qint32`.
    pub fn read_i32(&mut self) -> Option<i32> {
        self.take(4)
            .and_then(|bytes| <[u8; 4]>::try_from(bytes).ok())
            .map(i32::from_be_bytes)
    }

    /// `stream >> quint64`.
    pub fn read_u64(&mut self) -> Option<u64> {
        self.take(8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_be_bytes)
    }

    /// `stream >> qint64`.
    pub fn read_i64(&mut self) -> Option<i64> {
        self.take(8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(i64::from_be_bytes)
    }

    /// `stream >> quint8` / `qint8`.
    pub fn read_u8(&mut self) -> Option<u8> {
        self.take(1).map(|bytes| bytes[0])
    }

    /// `stream >> bool`.
    pub fn read_bool(&mut self) -> Option<bool> {
        self.read_u8().map(|value| value != 0)
    }

    /// `stream.readRawData(&value, N)` — `Serialize::read<MTP::AuthKey::Data>`.
    pub fn read_raw<const N: usize>(&mut self) -> Option<[u8; N]> {
        self.take(N).and_then(|bytes| <[u8; N]>::try_from(bytes).ok())
    }

    /// `stream >> QByteArray` — a null array reads back as empty.
    pub fn read_byte_array(&mut self) -> Option<Vec<u8>> {
        let length = self.read_u32()?;
        if length == NULL_LENGTH {
            return Some(Vec::new());
        }
        self.take(length as usize).map(<[u8]>::to_vec)
    }

    /// `stream >> Serialize::bytes(vector)` — a null length is an empty vector,
    /// and a short buffer clears it and marks the stream bad, as the C++ reader
    /// does after its 1 MiB steps.
    pub fn read_bytes(&mut self) -> Option<Vec<u8>> {
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
        if bytes.len() % 2 != 0 {
            self.fail();
            return None;
        }
        let units = bytes
            .chunks_exact(2)
            .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
            .collect::<Vec<u16>>();
        Some(String::from_utf16_lossy(&units))
    }

    /// `readColor`.
    pub fn read_color(&mut self) -> Option<Color> {
        self.read_u32().map(Color::from_packed)
    }

    /// The `dateTimeSize()` form of a `QDateTime`.
    pub fn read_date_time(&mut self) -> Option<SerializedDateTime> {
        let julian_day = self.read_i64()?;
        let msecs = self.read_u32()?;
        let spec = self
            .take(1)
            .map(|bytes| bytes[0] as i8)?;
        Some(SerializedDateTime {
            julian_day,
            msecs,
            spec,
        })
    }
}

/// The length prefix of a string, byte array or bytes vector.
pub const LENGTH_PREFIX_SIZE: usize = LENGTH_SIZE;