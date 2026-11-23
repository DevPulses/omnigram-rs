//! Exact payload sizes of the values the storage layer writes.
//!
//! Source of truth: the `*Size()` helpers of `storage/serialize_common.h`.
//! They are used to reserve one buffer up front, so every one has to match the
//! wire form byte for byte.

use crate::date_time::DATE_TIME_SIZE;

/// `sizeof(quint32)` — the length prefix of a string, a byte array or a bytes
/// vector.
pub const LENGTH_SIZE: usize = 4;

/// `stringSize` — a length prefix plus two bytes per UTF-16 code unit.
#[must_use]
pub fn string_size(value: &str) -> usize {
    LENGTH_SIZE + value.encode_utf16().count() * 2
}

/// `bytearraySize` — a length prefix plus the bytes.
#[must_use]
pub fn bytearray_size(value: &[u8]) -> usize {
    LENGTH_SIZE + value.len()
}

/// `bytesSize` — the same shape, used for `Serialize::bytes` vectors.
#[must_use]
pub fn bytes_size(value: &[u8]) -> usize {
    LENGTH_SIZE + value.len()
}

/// `colorSize` — one packed `quint32`.
#[must_use]
pub const fn color_size() -> usize {
    4
}

/// `dateTimeSize`.
#[must_use]
pub const fn date_time_size() -> usize {
    DATE_TIME_SIZE
}
