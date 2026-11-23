//! `ByteArrayWriter` — builds one storage buffer.
//!
//! Source of truth: `storage/serialize_common.{h,cpp}`.
//!
//! Every storage record is a `QDataStream` at version `Qt_5_1` with the default
//! big-endian order. This writer produces exactly that: numbers big-endian,
//! `bool` as one byte, `QString` as a UTF-16 code-unit count in bytes, byte
//! arrays length-prefixed, colours and date-times in their packed forms.

use crate::color::Color;
use crate::date_time::SerializedDateTime;
use crate::sizes::LENGTH_SIZE;

/// A null `QByteArray` / empty `Serialize::bytes` length.
pub const NULL_LENGTH: u32 = 0xFFFF_FFFF;

/// `Serialize::ByteArrayWriter`.
#[derive(Debug, Clone)]
pub struct ByteArrayWriter {
    data: Vec<u8>,
}

impl Default for ByteArrayWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl ByteArrayWriter {
    /// `ByteArrayWriter(int expectedSize = 0)`.
    #[must_use]
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// The same, with the buffer reserved for `expected_size` bytes.
    #[must_use]
    pub fn with_capacity(expected_size: usize) -> Self {
        Self {
            data: Vec::with_capacity(expected_size),
        }
    }

    /// `result() &&` — the bytes written so far.
    #[must_use]
    pub fn result(&self) -> &[u8] {
        &self.data
    }

    /// The bytes, consumed.
    #[must_use]
    pub fn into_result(self) -> Vec<u8> {
        self.data
    }

    /// `stream << quint32`.
    pub fn write_u32(&mut self, value: u32) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// `stream << qint32`.
    pub fn write_i32(&mut self, value: i32) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// `stream << quint64`.
    pub fn write_u64(&mut self, value: u64) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// `stream << qint64`.
    pub fn write_i64(&mut self, value: i64) -> &mut Self {
        self.data.extend_from_slice(&value.to_be_bytes());
        self
    }

    /// `stream << quint8` / `qint8`.
    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.data.push(value);
        self
    }

    /// `stream << bool` — one byte since Qt 5.0.
    pub fn write_bool(&mut self, value: bool) -> &mut Self {
        self.data.push(u8::from(value));
        self
    }

    /// `stream.writeRawData(bytes, size)` — no length prefix.
    pub fn write_raw(&mut self, bytes: &[u8]) -> &mut Self {
        self.data.extend_from_slice(bytes);
        self
    }

    /// `stream << QByteArray` — a null array is `0xFFFFFFFF`, an empty one is
    /// a zero length.
    pub fn write_byte_array(&mut self, value: Option<&[u8]>) -> &mut Self {
        match value {
            Some(bytes) => {
                self.write_u32(bytes.len() as u32);
                self.write_raw(bytes);
            }
            None => {
                self.write_u32(NULL_LENGTH);
            }
        }
        self
    }

    /// `stream << Serialize::bytes(vector)` — note that an *empty* vector is
    /// written as a null array, unlike a `QByteArray`.
    pub fn write_bytes(&mut self, value: &[u8]) -> &mut Self {
        if value.is_empty() {
            self.write_u32(NULL_LENGTH);
        } else {
            self.write_u32(value.len() as u32);
            self.write_raw(value);
        }
        self
    }

    /// `stream << QString` — a null string is `0xFFFFFFFF`.
    pub fn write_string(&mut self, value: Option<&str>) -> &mut Self {
        match value {
            Some(text) => {
                let units = text.encode_utf16().collect::<Vec<u16>>();
                self.write_u32((units.len() * 2) as u32);
                for unit in units {
                    self.data.extend_from_slice(&unit.to_be_bytes());
                }
            }
            None => {
                self.write_u32(NULL_LENGTH);
            }
        }
        self
    }

    /// `writeColor` — one packed `quint32`.
    pub fn write_color(&mut self, color: Color) -> &mut Self {
        self.write_u32(color.packed())
    }

    /// `dateTimeSize()` bytes of a `QDateTime`.
    pub fn write_date_time(&mut self, value: SerializedDateTime) -> &mut Self {
        self.write_i64(value.julian_day);
        self.write_u32(value.msecs);
        self.data.push(value.spec as u8);
        self
    }
}

/// `sizeof(quint32)`, re-exported where the length prefix is read or written.
pub const LENGTH_PREFIX_SIZE: usize = LENGTH_SIZE;