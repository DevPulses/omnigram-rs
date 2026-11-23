//! The whole settings map: a sequence of `uint32` block ids and payloads.
//!
//! Source of truth: the read loop of `Account::readSessionSettings` in
//! `storage/storage_account.cpp`.
//!
//! Reading stops at the first block that fails, and the whole settings read
//! then counts as failed — the C++ code returns a null settings pointer and
//! rewrites the file instead of half-applying it.

use crate::context::ReadSettingsContext;
use crate::read::read_setting;
use crate::sink::SettingsSink;
use crate::stream::StreamReader;

/// Reads every block of a decrypted settings map.
///
/// Returns `false` at the first unknown id, short payload or invalid legacy
/// value, exactly like the C++ loop.
pub fn read_settings_map(
    reader: &mut StreamReader<'_>,
    version: i32,
    context: &mut ReadSettingsContext,
    sink: &mut impl SettingsSink,
) -> bool {
    while !reader.at_end() {
        let Some(block_id) = reader.read_u32() else {
            return false;
        };
        if !read_setting(block_id, reader, version, context, sink) {
            return false;
        }
    }
    true
}