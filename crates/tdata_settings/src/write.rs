//! The write side of the settings map.
//!
//! Source of truth: `Account::writeSessionSettings` and `Account::writeMtpData`
//! in `storage/storage_account.cpp`.
//!
//! Both write the same shape the reader consumes: a `uint32` block id followed
//! by the block payload. The settings file holds the cache limits, the
//! serialized per-session settings when there are any, and the recent stickers
//! preload; the separate data file holds just the serialized MTP authorization.

use crate::blocks as id;
use crate::stream::StreamWriter;

/// The cache limits of an account, as `writeSessionSettings` stores them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CacheLimits {
    pub total_size: i64,
    pub total_time: i32,
    pub big_file_total_size: i64,
    pub big_file_total_time: i32,
}

/// Everything the settings file holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsSnapshot {
    pub cache: CacheLimits,
    /// `Main::SessionSettings::serialize()` — written only when non-empty.
    pub session_settings: Vec<u8>,
    /// `RecentStickerPreload` — document id to use count.
    pub recent_stickers: Vec<(u64, u16)>,
}

/// `writeSessionSettings` — the settings file payload, before encryption.
#[must_use]
pub fn write_settings_map(snapshot: &SettingsSnapshot) -> Vec<u8> {
    let mut out = StreamWriter::new();
    out.write_u32(id::DBI_CACHE_SETTINGS);
    out.write_i64(snapshot.cache.total_size);
    out.write_i32(snapshot.cache.total_time);
    out.write_i64(snapshot.cache.big_file_total_size);
    out.write_i32(snapshot.cache.big_file_total_time);
    if !snapshot.session_settings.is_empty() {
        out.write_u32(id::DBI_SESSION_SETTINGS);
        out.write_byte_array(&snapshot.session_settings);
    }
    out.write_u32(id::DBI_RECENT_STICKERS);
    out.write_pairs_u64_u16(&snapshot.recent_stickers);
    out.into_data()
}

/// `writeMtpData` — the separate data-file payload, before encryption.
#[must_use]
pub fn write_mtp_data(serialized: &[u8]) -> Vec<u8> {
    let mut out = StreamWriter::new();
    out.write_u32(id::DBI_MTP_AUTHORIZATION);
    out.write_byte_array(serialized);
    out.into_data()
}