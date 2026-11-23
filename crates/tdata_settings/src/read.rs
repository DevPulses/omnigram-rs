//! `ReadSetting` — one block of the settings file at a time.
//!
//! Source of truth: `ReadSetting` in
//! `storage/details/storage_settings_scheme.cpp`.
//!
//! Each block is a `uint32` id followed by that block's payload. The function
//! reads exactly the payload, leaves the context filled and reports the values
//! the C++ code would have written as [`LegacyEffect`]s. An unknown id, a
//! truncated payload or an invalid legacy value fails the whole settings read,
//! exactly as the C++ `false` return does.

use crate::blocks as id;
use crate::context::{LegacyAuthKey, LegacyDcEndpoint, LegacyDcOptions, ReadSettingsContext, LEGACY_AUTH_KEY_SIZE};
use crate::sink::{
    CallSettings, LegacyEffect, ProxyData, ProxySettings, ProxyType, SettingsSink, WindowPosition,
};
use crate::stream::StreamReader;

/// `kMaxFileInMemory` — the smallest cache limit that makes sense, mirrored by
/// `Database::Settings().maxDataSize`.
pub const MAX_DATA_SIZE: i64 = 10 * 1024 * 1024;

/// `style::kScaleAuto` — the "let the system decide" scale.
pub const SCALE_AUTO: i32 = 0;

/// The file driver prefix of the settings-file magic: the version at which the
/// tile flag stopped being written unconditionally.
pub const TILE_LEGACY_FIX_VERSION: i32 = 8005;

/// `NoTimeLimit` — the workaround for the broken cache time limit storage:
/// zero, `int32` max and the truncated `int64` max all mean "no limit".
#[must_use]
pub fn no_time_limit(stored: i32) -> bool {
    stored == 0 || stored == i32::MAX || stored == -1
}

/// `ReadSetting` — reads one block payload into `context` and `sink`.
///
/// Returns `false` exactly where the C++ code does: an unknown block id, a
/// payload shorter than the block needs, or a legacy value outside its range.
pub fn read_setting(
    block_id: u32,
    reader: &mut StreamReader<'_>,
    version: i32,
    context: &mut ReadSettingsContext,
    sink: &mut impl SettingsSink,
) -> bool {
    match block_id {
        id::DBI_DC_OPTION_OLD_OLD => {
            let (Some(dc_id), Some(_host), Some(ip), Some(port)) = (
                reader.read_u32(),
                reader.read_string(),
                reader.read_string(),
                reader.read_u32(),
            ) else {
                return false;
            };
            let endpoints = legacy_dc_endpoints(context);
            endpoints.push(LegacyDcEndpoint {
                dc_id,
                flags: 0,
                ip,
                port,
            });
            context.legacy_read = true;
        }

        id::DBI_DC_OPTION_OLD => {
            let (Some(dc_id), Some(flags), Some(ip), Some(port)) = (
                reader.read_u32(),
                reader.read_i32(),
                reader.read_string(),
                reader.read_u32(),
            ) else {
                return false;
            };
            let endpoints = legacy_dc_endpoints(context);
            endpoints.push(LegacyDcEndpoint {
                dc_id,
                flags,
                ip,
                port,
            });
            context.legacy_read = true;
        }

        id::DBI_DC_OPTIONS_OLD => {
            // `constructFromSerialized` parses the blob into a live DcOptions;
            // that store is mtproto's area, so the bytes travel onwards.
            let Some(serialized) = reader.read_byte_array() else {
                return false;
            };
            context.fallback_config_legacy_dc_options = LegacyDcOptions::Serialized(serialized);
            context.legacy_read = true;
        }

        id::DBI_APPLICATION_SETTINGS => {
            let Some(serialized) = reader.read_byte_array() else {
                return false;
            };
            sink.apply(LegacyEffect::ApplicationSettings { serialized, version });
        }

        id::DBI_CHAT_SIZE_MAX_OLD => {
            let Some(value) = reader.read_i32() else {
                return false;
            };
            context.fallback_config_legacy_chat_size_max = value;
            context.legacy_read = true;
        }

        id::DBI_SAVED_GIFS_LIMIT_OLD => {
            let Some(value) = reader.read_i32() else {
                return false;
            };
            context.fallback_config_legacy_saved_gifs_limit = value;
            context.legacy_read = true;
        }

        id::DBI_STICKERS_RECENT_LIMIT_OLD => {
            let Some(value) = reader.read_i32() else {
                return false;
            };
            context.fallback_config_legacy_stickers_recent_limit = value;
            context.legacy_read = true;
        }

        id::DBI_STICKERS_FAVED_LIMIT_OLD => {
            let Some(value) = reader.read_i32() else {
                return false;
            };
            context.fallback_config_legacy_stickers_faved_limit = value;
            context.legacy_read = true;
        }

        id::DBI_MEGAGROUP_SIZE_MAX_OLD => {
            let Some(value) = reader.read_i32() else {
                return false;
            };
            context.fallback_config_legacy_megagroup_size_max = value;
            context.legacy_read = true;
        }

        id::DBI_USER => {
            let (Some(user_id), Some(dc_id)) = (reader.read_i32(), reader.read_u32()) else {
                return false;
            };
            context.mtp_legacy_user_id = user_id;
            context.mtp_legacy_main_dc_id = dc_id as i32;
        }

        id::DBI_KEY => {
            let Some(dc_id) = reader.read_i32() else {
                return false;
            };
            let Some(key) = reader.read_raw::<LEGACY_AUTH_KEY_SIZE>() else {
                return false;
            };
            context.mtp_legacy_keys.push(LegacyAuthKey { dc_id, key });
        }

        id::DBI_MTP_AUTHORIZATION => {
            let Some(serialized) = reader.read_byte_array() else {
                return false;
            };
            context.mtp_authorization = serialized;
        }

        id::DBI_AUTO_START => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::AutoStart(value == 1));
        }

        id::DBI_START_MINIMIZED => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::StartMinimized(value == 1));
        }

        id::DBI_SEND_TO_MENU => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::SendToMenu(value == 1));
        }

        id::DBI_USE_EXTERNAL_VIDEO_PLAYER_OLD => {
            if read_i32(reader).is_none() {
                return false;
            }
        }

        id::DBI_CACHE_SETTINGS_OLD => {
            let (Some(size), Some(time)) = (reader.read_i64(), reader.read_i32()) else {
                return false;
            };
            if size <= MAX_DATA_SIZE || (!no_time_limit(time) && time < 0) {
                return false;
            }
            let time = if no_time_limit(time) { 0 } else { time };
            context.cache_total_size_limit = size;
            context.cache_total_time_limit = time;
            context.cache_big_file_total_size_limit = size;
            context.cache_big_file_total_time_limit = time;
            context.legacy_read = true;
        }

        id::DBI_CACHE_SETTINGS => {
            let (Some(size), Some(time), Some(size_big), Some(time_big)) =
                (reader.read_i64(), reader.read_i32(), reader.read_i64(), reader.read_i32())
            else {
                return false;
            };
            if size <= MAX_DATA_SIZE
                || size_big <= MAX_DATA_SIZE
                || (!no_time_limit(time) && time < 0)
                || (!no_time_limit(time_big) && time_big < 0)
            {
                return false;
            }
            context.cache_total_size_limit = size;
            context.cache_total_time_limit = if no_time_limit(time) { 0 } else { time };
            context.cache_big_file_total_size_limit = size_big;
            context.cache_big_file_total_time_limit =
                if no_time_limit(time_big) { 0 } else { time_big };
        }

        id::DBI_POWER_SAVING => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::PowerSaving(value));
        }

        id::DBI_SOUND_FLASH_BOUNCE_NOTIFY_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::SoundNotify(value & 0x01 == 0x01));
            sink.apply(LegacyEffect::FlashBounceNotify(value & 0x02 == 0x00));
            context.legacy_read = true;
        }

        id::DBI_AUTO_DOWNLOAD_OLD => {
            let (Some(photo), Some(audio), Some(gif)) =
                (reader.read_i32(), reader.read_i32(), reader.read_i32())
            else {
                return false;
            };
            sink.apply(LegacyEffect::AutoDownloadMasks { photo, audio, gif });
            context.legacy_read = true;
        }

        id::DBI_AUTO_PLAY_OLD => {
            let Some(gif) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::AutoPlay(gif != 0));
            context.legacy_read = true;
        }

        id::DBI_DIALOGS_MODE_OLD => {
            if reader.read_i32().is_none() || reader.read_i32().is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_DIALOGS_FILTERS_OLD => {
            let Some(enabled) = read_i32(reader) else {
                return false;
            };
            context.session_settings.dialogs_filters_enabled = Some(enabled == 1);
            context.legacy_read = true;
        }

        id::DBI_MODERATE_MODE_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::ModerateModeEnabled(value == 1));
            context.legacy_read = true;
        }

        id::DBI_INCLUDE_MUTED_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::IncludeMutedCounter(value == 1));
            context.legacy_read = true;
        }

        id::DBI_SHOWING_SAVED_GIFS_OLD => {
            if read_i32(reader).is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_DESKTOP_NOTIFY_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::DesktopNotify(value == 1));
            context.legacy_read = true;
        }

        id::DBI_WINDOWS_NOTIFICATIONS_OLD => {
            if read_i32(reader).is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_NATIVE_NOTIFICATIONS_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::NativeNotifications(value == 1));
            context.legacy_read = true;
        }

        id::DBI_NOTIFICATIONS_COUNT_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::NotificationsCount(if value > 0 { value } else { 3 }));
            context.legacy_read = true;
        }

        id::DBI_NOTIFICATIONS_CORNER_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            let corner = if (0..4).contains(&value) { value } else { 2 };
            sink.apply(LegacyEffect::NotificationsCorner(corner));
            context.legacy_read = true;
        }

        id::DBI_DIALOGS_WIDTH_RATIO_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::DialogsWidthRatio(value as f64 / 1_000_000.0));
            context.legacy_read = true;
        }

        id::DBI_LAST_SEEN_WARNING_SEEN_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::LastSeenWarningSeen(value == 1));
            context.legacy_read = true;
        }

        id::DBI_SESSION_SETTINGS => {
            let Some(serialized) = reader.read_byte_array() else {
                return false;
            };
            context.session_settings.serialized = Some((serialized, version));
        }

        id::DBI_WORK_MODE_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            // WorkMode: WindowAndTray 0, TrayOnly 1, WindowOnly 2.
            let mode = if value == 1 || value == 2 { value } else { 0 };
            sink.apply(LegacyEffect::WorkMode(mode));
            context.legacy_read = true;
        }

        id::DBI_TXT_DOMAIN_STRING_OLD_OLD => {
            if reader.read_string().is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_TXT_DOMAIN_STRING_OLD => {
            let Some(value) = reader.read_string() else {
                return false;
            };
            context.fallback_config_legacy_txt_domain_string = value;
            context.legacy_read = true;
        }

        id::DBI_CONNECTION_TYPE_OLD_OLD => {
            let Some(connection_type) = read_i32(reader) else {
                return false;
            };
            let mut proxy = ProxyData::default();
            if connection_type == CT_HTTP_PROXY || connection_type == CT_TCP_PROXY {
                let (Some(host), Some(port), Some(user), Some(password)) = (
                    reader.read_string(),
                    reader.read_i32(),
                    reader.read_string(),
                    reader.read_string(),
                ) else {
                    return false;
                };
                proxy.host = host;
                proxy.user = user;
                proxy.password = password;
                proxy.port = port as u32;
                proxy.kind = Some(if connection_type == CT_TCP_PROXY {
                    ProxyType::Socks5
                } else {
                    ProxyType::Http
                });
            }
            let selected = if proxy.is_set() { Some(proxy.clone()) } else { None };
            let list = selected.clone().into_iter().collect::<Vec<_>>();
            sink.apply(LegacyEffect::ProxySelection {
                list,
                selected,
                settings: if proxy.is_set() {
                    ProxySettings::Enabled
                } else {
                    ProxySettings::System
                },
                use_for_calls: None,
                try_ipv6: None,
            });
            context.legacy_read = true;
        }

        id::DBI_CONNECTION_TYPE_OLD => {
            let Some(connection_type) = read_i32(reader) else {
                return false;
            };
            if connection_type == CT_PROXIES_LIST_OLD || connection_type == CT_PROXIES_LIST {
                let (Some(count), Some(mut index)) = (reader.read_i32(), reader.read_i32()) else {
                    return false;
                };
                let mut settings = 0i32;
                let mut calls = 0i32;
                if connection_type == CT_PROXIES_LIST {
                    let (Some(stored_settings), Some(stored_calls)) =
                        (reader.read_i32(), reader.read_i32())
                    else {
                        return false;
                    };
                    settings = stored_settings;
                    calls = stored_calls;
                } else if index.abs() > count {
                    calls = 1;
                    index -= if index > 0 { count } else { -count };
                }

                let mut list = Vec::new();
                for _ in 0..count {
                    let Some(proxy) = read_proxy(reader) else {
                        return false;
                    };
                    if proxy.is_set() {
                        list.push(proxy);
                    } else if i64::from(index) < -(list.len() as i64) {
                        index += 1;
                    } else if i64::from(index) > list.len() as i64 {
                        index -= 1;
                    }
                }
                if !reader.is_ok() {
                    return false;
                }

                if connection_type == CT_PROXIES_LIST_OLD {
                    settings = if index > 0 && index as usize <= list.len() {
                        ProxySettings::Enabled as i32
                    } else {
                        ProxySettings::System as i32
                    };
                    index = index.abs();
                }
                let selected = if index > 0 && index as usize <= list.len() {
                    Some(list[index as usize - 1].clone())
                } else {
                    None
                };
                let settings = match settings {
                    value if value == ProxySettings::Enabled as i32 => {
                        if selected.is_some() {
                            ProxySettings::Enabled
                        } else {
                            ProxySettings::System
                        }
                    }
                    value if value == ProxySettings::Disabled as i32 => ProxySettings::Disabled,
                    value if value == ProxySettings::System as i32 => ProxySettings::System,
                    _ => ProxySettings::System,
                };
                sink.apply(LegacyEffect::ProxySelection {
                    list,
                    selected,
                    settings,
                    use_for_calls: Some(calls == 1),
                    try_ipv6: None,
                });
            } else {
                let Some(proxy) = read_proxy(reader) else {
                    return false;
                };
                if proxy.is_set() {
                    let settings = if connection_type == CT_TCP_PROXY
                        || connection_type == CT_HTTP_PROXY
                    {
                        ProxySettings::Enabled
                    } else {
                        ProxySettings::System
                    };
                    sink.apply(LegacyEffect::ProxySelection {
                        list: vec![proxy.clone()],
                        selected: Some(proxy),
                        settings,
                        use_for_calls: None,
                        try_ipv6: None,
                    });
                } else {
                    sink.apply(LegacyEffect::ProxySelection {
                        list: Vec::new(),
                        selected: None,
                        settings: ProxySettings::System,
                        use_for_calls: None,
                        try_ipv6: None,
                    });
                }
            }
            context.legacy_read = true;
        }

        id::DBI_THEME_KEY_OLD => {
            let Some(key) = reader.read_u64() else {
                return false;
            };
            context.theme_key_legacy = key;
            context.legacy_read = true;
        }

        id::DBI_THEME_KEY => {
            let (Some(key_day), Some(key_night), Some(night_mode)) =
                (reader.read_u64(), reader.read_u64(), reader.read_u32())
            else {
                return false;
            };
            context.theme_key_day = key_day;
            context.theme_key_night = key_night;
            // The theme store itself is outside this task; the pass keeps the
            // value it just established, which `dbiTileBackgroundOld` reads.
            context.night_mode = night_mode == 1;
            sink.apply(LegacyEffect::ThemeNightMode(night_mode == 1));
        }

        id::DBI_BACKGROUND_KEY => {
            let (Some(key_day), Some(key_night)) = (reader.read_u64(), reader.read_u64()) else {
                return false;
            };
            context.background_key_day = key_day;
            context.background_key_night = key_night;
            context.background_keys_read = true;
        }

        id::DBI_LANG_PACK_KEY => {
            let Some(key) = reader.read_u64() else {
                return false;
            };
            context.lang_pack_key = key;
        }

        id::DBI_LANGUAGES_KEY => {
            let Some(key) = reader.read_u64() else {
                return false;
            };
            context.languages_key = key;
        }

        id::DBI_TRY_IPV6_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::ProxyTryIPv6(value == 1));
            context.legacy_read = true;
        }

        id::DBI_SEEN_TRAY_TOOLTIP => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::SeenTrayTooltip(value == 1));
        }

        id::DBI_AUTO_UPDATE => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::AutoUpdate(value == 1));
        }

        id::DBI_LAST_UPDATE_CHECK => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::LastUpdateCheck(value));
        }

        id::DBI_SCALE_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            let scale = match value {
                0 => SCALE_AUTO,
                1 => 100,
                2 => 125,
                3 => 150,
                4 => 200,
                _ => context.scale_from_command_line.unwrap_or(SCALE_AUTO),
            };
            sink.apply(LegacyEffect::Scale(scale));
            context.legacy_read = true;
        }

        id::DBI_SCALE_PERCENT => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            // A command-line scale wins over the stored one.
            if context.scale_from_command_line.unwrap_or(SCALE_AUTO) == SCALE_AUTO {
                sink.apply(LegacyEffect::Scale(value));
            }
        }

        id::DBI_LANG_OLD => {
            if read_i32(reader).is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_LANG_FILE_OLD => {
            if reader.read_string().is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_WINDOW_POSITION_OLD => {
            let (Some(x), Some(y), Some(w), Some(h), Some(moncrc), Some(maximized)) = (
                reader.read_i32(),
                reader.read_i32(),
                reader.read_i32(),
                reader.read_i32(),
                reader.read_u64(),
                reader.read_bool(),
            ) else {
                return false;
            };
            sink.apply(LegacyEffect::WindowPosition(WindowPosition {
                x,
                y,
                w,
                h,
                moncrc,
                maximized,
            }));
            context.legacy_read = true;
        }

        id::DBI_LOGGED_PHONE_NUMBER_OLD => {
            if reader.read_string().is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_MUTE_PEER_OLD => {
            if reader.read_u64().is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_MUTED_PEERS_OLD => {
            let Some(count) = reader.read_u32() else {
                return false;
            };
            for _ in 0..count {
                if reader.read_u64().is_none() {
                    return false;
                }
            }
            context.legacy_read = true;
        }

        id::DBI_SEND_KEY_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            // InputSubmitSettings: Enter 0, CtrlEnter 1, Both 2, None 3.
            if value != 0 && value != 1 {
                return false;
            }
            sink.apply(LegacyEffect::SendSubmitWay(value));
            context.legacy_read = true;
        }

        id::DBI_CATS_AND_DOGS_OLD => {
            if read_i32(reader).is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_TILE_BACKGROUND_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            let tile = if version < TILE_LEGACY_FIX_VERSION
                && !context.legacy_has_custom_day_background
            {
                false
            } else {
                value == 1
            };
            if context.night_mode {
                context.tile_day = tile;
            } else {
                context.tile_night = tile;
            }
            context.tile_read = true;
            context.legacy_read = true;
        }

        id::DBI_TILE_BACKGROUND => {
            let (Some(tile_day), Some(tile_night)) = (reader.read_i32(), reader.read_i32()) else {
                return false;
            };
            context.tile_day = tile_day != 0;
            context.tile_night = tile_night != 0;
            context.tile_read = true;
        }

        id::DBI_ADAPTIVE_FOR_WIDE_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::AdaptiveForWide(value == 1));
            context.legacy_read = true;
        }

        id::DBI_AUTO_LOCK_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::AutoLock(value));
            context.legacy_read = true;
        }

        id::DBI_REPLACE_EMOJI_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::ReplaceEmoji(value == 1));
            context.legacy_read = true;
        }

        id::DBI_SUGGEST_EMOJI_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::SuggestEmoji(value == 1));
            context.legacy_read = true;
        }

        id::DBI_SUGGEST_STICKERS_BY_EMOJI_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::SuggestStickersByEmoji(value == 1));
            context.legacy_read = true;
        }

        id::DBI_DEFAULT_ATTACH => {
            if read_i32(reader).is_none() {
                return false;
            }
        }

        id::DBI_NOTIFY_VIEW_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            // NotifyView: ShowPreview 0, ShowName 1, ShowNothing 2.
            let view = if value == 1 || value == 2 { value } else { 0 };
            sink.apply(LegacyEffect::NotifyView(view));
            context.legacy_read = true;
        }

        id::DBI_ASK_DOWNLOAD_PATH_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::AskDownloadPath(value == 1));
            context.legacy_read = true;
        }

        id::DBI_DOWNLOAD_PATH_OLD_OLD => {
            let Some(value) = reader.read_string() else {
                return false;
            };
            sink.apply(LegacyEffect::DownloadPath {
                path: normalize_download_path(value),
                bookmark: Vec::new(),
            });
            context.legacy_read = true;
        }

        id::DBI_DOWNLOAD_PATH_OLD => {
            let (Some(value), Some(bookmark)) = (reader.read_string(), reader.read_byte_array())
            else {
                return false;
            };
            sink.apply(LegacyEffect::DownloadPath {
                path: normalize_download_path(value),
                bookmark,
            });
            context.legacy_read = true;
        }

        id::DBI_COMPRESS_PASTED_IMAGE_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::SendFilesWay {
                group_files: value == 1,
            });
            context.legacy_read = true;
        }

        id::DBI_EMOJI_TAB_OLD => {
            if read_i32(reader).is_none() {
                return false;
            }
            context.legacy_read = true;
        }

        id::DBI_RECENT_EMOJI_OLD_OLD_OLD => {
            let Some(items) = reader.read_pairs_u32_u16() else {
                return false;
            };
            if !items.is_empty() {
                let items = items.into_iter().map(|(key, used)| (u64::from(key), used)).collect();
                sink.apply(LegacyEffect::LegacyRecentEmoji(items));
            }
            context.legacy_read = true;
        }

        id::DBI_RECENT_EMOJI_OLD_OLD => {
            let Some(items) = reader.read_pairs_u64_u16() else {
                return false;
            };
            if !items.is_empty() {
                sink.apply(LegacyEffect::LegacyRecentEmoji(items));
            }
            context.legacy_read = true;
        }

        id::DBI_RECENT_EMOJI_OLD => {
            let Some(items) = reader.read_pairs_string_u16() else {
                return false;
            };
            sink.apply(LegacyEffect::LegacyRecentEmojiById(items));
            context.legacy_read = true;
        }

        id::DBI_RECENT_STICKERS => {
            let Some(items) = reader.read_pairs_u64_u16() else {
                return false;
            };
            sink.apply(LegacyEffect::RecentStickersPreload(items));
        }

        id::DBI_EMOJI_VARIANTS_OLD_OLD => {
            let Some(items) = reader.read_map_u32_u64() else {
                return false;
            };
            let items = items
                .into_iter()
                .map(|(key, value)| (u64::from(key), value))
                .collect();
            sink.apply(LegacyEffect::LegacyEmojiVariants(items));
            context.legacy_read = true;
        }

        id::DBI_EMOJI_VARIANTS_OLD => {
            let Some(items) = reader.read_map_string_i32() else {
                return false;
            };
            sink.apply(LegacyEffect::LegacyEmojiVariantsById(items));
            context.legacy_read = true;
        }

        id::DBI_HIDDEN_PINNED_MESSAGES_OLD => {
            let Some(items) = reader.read_map_u64_i32() else {
                return false;
            };
            context.session_settings.hidden_pinned_messages.extend(items);
            context.legacy_read = true;
        }

        id::DBI_DIALOG_LAST_PATH => {
            let Some(path) = reader.read_string() else {
                return false;
            };
            sink.apply(LegacyEffect::DialogLastPath(path));
        }

        id::DBI_SONG_VOLUME_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::SongVolume(volume(value)));
            context.legacy_read = true;
        }

        id::DBI_VIDEO_VOLUME_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            sink.apply(LegacyEffect::VideoVolume(volume(value)));
            context.legacy_read = true;
        }

        id::DBI_PLAYBACK_SPEED_OLD => {
            let Some(value) = read_i32(reader) else {
                return false;
            };
            if value == 2 {
                sink.apply(LegacyEffect::PlaybackSpeed(2.0));
            }
            context.legacy_read = true;
        }

        id::DBI_CALL_SETTINGS_OLD => {
            let Some(serialized) = reader.read_byte_array() else {
                return false;
            };
            let mut nested = StreamReader::new(&serialized);
            let parsed = (
                nested.read_string(),
                nested.read_i32(),
                nested.read_string(),
                nested.read_i32(),
                nested.read_i32(),
            );
            if let (Some(output_device_id), Some(output_volume), Some(input_device_id), Some(input_volume), Some(ducking_enabled)) = parsed {
                if nested.is_ok() {
                    sink.apply(LegacyEffect::CallSettings(CallSettings {
                        output_device_id,
                        output_volume,
                        input_device_id,
                        input_volume,
                        ducking_enabled,
                    }));
                }
            }
            context.legacy_read = true;
        }

        id::DBI_FALLBACK_PRODUCTION_CONFIG => {
            let Some(serialized) = reader.read_byte_array() else {
                return false;
            };
            context.fallback_config = serialized;
        }

        _ => return false,
    }

    reader.is_ok()
}

/// `dbictHttpProxy` / `dbictTcpProxy` of the oldest connection block.
const CT_HTTP_PROXY: i32 = 2;
const CT_TCP_PROXY: i32 = 3;
/// `dbictProxiesListOld` / `dbictProxiesList` of the newer legacy block.
const CT_PROXIES_LIST_OLD: i32 = 4;
const CT_PROXIES_LIST: i32 = 5;
/// `legacyProxyTypeShift` — the offset old builds added to the proxy type.
const LEGACY_PROXY_TYPE_SHIFT: i32 = 1024;

/// `stream >> qint32`, without repeating the `else { return false }` dance.
fn read_i32(reader: &mut StreamReader<'_>) -> Option<i32> {
    reader.read_i32()
}

/// `readProxy` of `dbiConnectionTypeOld`.
fn read_proxy(reader: &mut StreamReader<'_>) -> Option<ProxyData> {
    let (proxy_type, host, port, user, password) = (
        reader.read_i32()?,
        reader.read_string()?,
        reader.read_i32()?,
        reader.read_string()?,
        reader.read_string()?,
    );
    let kind = if proxy_type == CT_TCP_PROXY {
        ProxyType::Socks5
    } else if proxy_type == CT_HTTP_PROXY {
        ProxyType::Http
    } else if proxy_type == LEGACY_PROXY_TYPE_SHIFT + 1 {
        ProxyType::Socks5
    } else if proxy_type == LEGACY_PROXY_TYPE_SHIFT + 2 {
        ProxyType::Http
    } else if proxy_type == LEGACY_PROXY_TYPE_SHIFT + 3 {
        ProxyType::Mtproto
    } else {
        ProxyType::None
    };
    Some(ProxyData {
        kind: Some(kind),
        host,
        port: port as u32,
        user,
        password,
    })
}

/// The endpoint list of the legacy dc options, starting it if needed.
fn legacy_dc_endpoints(context: &mut ReadSettingsContext) -> &mut Vec<LegacyDcEndpoint> {
    if let LegacyDcOptions::Serialized(_) = context.fallback_config_legacy_dc_options {
        // `constructAddOne` after a serialized blob appends to the parsed list
        // in C++; without that store the blob is dropped, which cannot happen
        // in a file written by one process.
        context.fallback_config_legacy_dc_options = LegacyDcOptions::Empty;
    }
    if let LegacyDcOptions::Empty = context.fallback_config_legacy_dc_options {
        context.fallback_config_legacy_dc_options = LegacyDcOptions::Endpoints(Vec::new());
    }
    match &mut context.fallback_config_legacy_dc_options {
        LegacyDcOptions::Endpoints(endpoints) => endpoints,
        LegacyDcOptions::Serialized(_) | LegacyDcOptions::Empty => {
            unreachable!("the list was just ensured")
        }
    }
}

/// `std::clamp(value / 1e6, 0., 1.)`.
fn volume(value: i32) -> f64 {
    (value as f64 / 1e6).clamp(0.0, 1.0)
}

/// The legacy download path gains a trailing slash, like the C++ block does.
fn normalize_download_path(path: String) -> String {
    if !path.is_empty() && !path.ends_with('/') {
        path + "/"
    } else {
        path
    }
}