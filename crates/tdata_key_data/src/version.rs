//! Application version stored in, and checked against, every `TDF$` file.
//!
//! Source of truth: `tdesktop/Telegram/build/version` (`AppVersion 7004002`),
//! mirrored by `core/version.h`.

/// `AppVersion` — the value written into every container header and fed into
/// the file signature (`storage_file_utilities.cpp`, `writeHeader` / `finish`).
///
/// A container whose recorded version is *greater* than this one is refused by
/// `ReadFile`, and the signature check feeds this value in, so bumping it
/// invalidates every existing file unless the same number is read back.
pub const APP_VERSION: i32 = 7_004_002;