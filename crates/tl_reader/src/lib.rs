//! Чтение TL-потока: примитив и базовые типы.
//!
//! Соответствие оригиналу:
//! - `reader.rs` — `Reader<mtpPrime>` из `mtproto/core_types.h`;
//! - `basic_types.rs` — `read`-половина `lib_tl/tl/tl_basic_types.h`.

pub mod basic_types;
pub mod reader;

pub use basic_types::{
    read_double, read_int, read_int128, read_int256, read_long, read_string, read_vector, Int128,
    Int256, ID_BYTES, ID_DOUBLE, ID_FLAGS, ID_FLAGS64, ID_INT, ID_INT128, ID_INT256, ID_LONG,
    ID_STRING, ID_VECTOR,
};
pub use reader::{Eof, Reader};
