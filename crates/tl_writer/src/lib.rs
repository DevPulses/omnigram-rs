//! Запись TL-потока: примитив и базовые типы.
//!
//! Соответствие оригиналу:
//! - `writer.rs` — `Writer<mtpBuffer>` из `mtproto/core_types.h`
//!   и `Writer<LengthCounter>` из `lib_tl/tl/tl_basic_types.h`;
//! - `basic_types.rs` — `write`-половина `lib_tl/tl/tl_basic_types.h`.

pub mod basic_types;
pub mod writer;

pub use writer::{count_bytes, Writer};

pub use basic_types::{
    write_boxed, write_double, write_int, write_int128, write_int256, write_long, write_string,
    write_vector, write_vector_int,
};

// Константы-конструкторы общие с ридером (единая библиотека `tl` в C++).
pub use tl_reader::{
    ID_BYTES, ID_DOUBLE, ID_FLAGS, ID_FLAGS64, ID_INT, ID_INT128, ID_INT256, ID_LONG, ID_STRING,
    ID_VECTOR,
};
