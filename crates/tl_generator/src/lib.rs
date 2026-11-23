//! Генератор Rust-кода из `.tl`-схем Telegram.
//!
//! Порт `read-write`-секции `lib_tl/tl/generate_tl.py` (кодогенератор
//! scheme.tl → scheme.h/cpp в tdesktop). Источник правды — оригинальный
//! скрипт; оракул — сверка вычисленных CRC-32 конструкторов с
//! аннотированными id в `api.tl`/`mtproto.tl`.

pub mod crc32;
pub mod rust_gen;
pub mod generated_mtproto;
pub mod parser;

pub use rust_gen::generate;
pub use parser::{parse_files, Scheme};
