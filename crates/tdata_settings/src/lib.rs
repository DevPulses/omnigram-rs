//! The account settings file: block ids, wire format and the read scheme.
//!
//! Source of truth: `tdesktop/Telegram/SourceFiles/storage/details/storage_settings_scheme.{h,cpp}`
//! and the settings loop of `storage/storage_account.cpp`.
//!
//! A settings file is one encrypted descriptor holding a flat sequence of
//! `uint32` block ids, each followed by that block's payload in the
//! `QDataStream` format of Qt 5.1. Blocks are permanent: an unknown id fails
//! the whole read, so the ids in [`blocks`] are format, not convention.
//!
//! Layers:
//! - [`stream`] — the `QDataStream` primitives the payloads are built from.
//! - [`blocks`] — every `dbi*` id the scheme knows.
//! - [`context`] — `ReadSettingsContext`, the values collected for later use.
//! - [`read`] — `ReadSetting`, one block at a time.
//! - [`map`] — the loop over all blocks of one file.
//! - [`apply`] — `ApplyReadFallbackConfig`.
//! - [`sink`] — the effects that belong to the application settings, the theme
//!   or the platform, reported in file order instead of being written here.
//! - [`write`] — `writeSessionSettings` / `writeMtpData`, the same blocks out.

pub mod apply;
pub mod blocks;
pub mod context;
pub mod map;
pub mod read;
pub mod sink;
pub mod stream;
pub mod write;

pub use apply::{apply_read_fallback_config, FallbackConfigAction, LegacyFallbackConfig};
pub use context::{
    LegacyAuthKey, LegacyDcEndpoint, LegacyDcOptions, ReadSettingsContext, SessionSettingsInput,
};
pub use map::read_settings_map;
pub use read::{no_time_limit, read_setting, MAX_DATA_SIZE, SCALE_AUTO};
pub use sink::{
    CallSettings, LegacyEffect, ProxyData, ProxySettings, ProxyType, RecordingSink, SettingsSink,
    WindowPosition,
};
pub use stream::{StreamReader, StreamWriter};
pub use write::{write_mtp_data, write_settings_map, CacheLimits, SettingsSnapshot};
