//! The `QDataStream` primitives every storage record is built from.
//!
//! Source of truth: `tdesktop/Telegram/SourceFiles/storage/serialize_common.{h,cpp}`.
//!
//! A storage record is one buffer produced by `ByteArrayWriter` and read back by
//! `ByteArrayReader`: `QDataStream` at version `Qt_5_1`, big-endian, with the
//! packed forms for colours and date-times and the null-length conventions of
//! `QByteArray` and `Serialize::bytes`.
//!
//! Layers:
//! - [`writer`] — `ByteArrayWriter`.
//! - [`reader`] — `ByteArrayReader`, with the quiet default-on-failure rule.
//! - [`sizes`] — the `*Size()` helpers used to reserve a buffer up front.
//! - [`color`] — the packed colour value.
//! - [`date_time`] — the three-field `QDateTime` shape.

pub mod color;
pub mod date_time;
pub mod reader;
pub mod sizes;
pub mod writer;

pub use color::Color;
pub use date_time::{SerializedDateTime, DATE_TIME_SIZE, LOCAL_TIME, OFFSET_FROM_UTC, TIME_ZONE, UTC};
pub use reader::{ByteArrayReader, LENGTH_PREFIX_SIZE};
pub use sizes::{bytearray_size, bytes_size, color_size, date_time_size, string_size, LENGTH_SIZE};
pub use writer::{ByteArrayWriter, NULL_LENGTH};
