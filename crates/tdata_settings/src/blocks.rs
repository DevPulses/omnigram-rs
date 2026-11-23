//! Block ids of the account settings file.
//!
//! Source of truth: the `enum` in
//! `tdesktop/Telegram/SourceFiles/storage/details/storage_settings_scheme.h`.
//!
//! A settings file is a flat sequence of `uint32` block ids, each followed by
//! that block's payload. Ids are permanent: the reserved gaps and the legacy
//! `*Old` names are part of the format, not a naming accident.

/// `dbiKey` — a legacy MTP authorization key with its dc id.
pub const DBI_KEY: u32 = 0x00;
/// `dbiUser` — a legacy user id with its dc id.
pub const DBI_USER: u32 = 0x01;
/// `dbiDcOptionOldOld` — the oldest single dc endpoint.
pub const DBI_DC_OPTION_OLD_OLD: u32 = 0x02;
/// `dbiChatSizeMaxOld` — a legacy fallback config limit.
pub const DBI_CHAT_SIZE_MAX_OLD: u32 = 0x03;
/// `dbiMutePeerOld` — deprecated.
pub const DBI_MUTE_PEER_OLD: u32 = 0x04;
/// `dbiSendKeyOld` — the legacy Enter / Ctrl+Enter submit choice.
pub const DBI_SEND_KEY_OLD: u32 = 0x05;
/// `dbiAutoStart` — the current autostart flag.
pub const DBI_AUTO_START: u32 = 0x06;
/// `dbiStartMinimized` — the current "start minimized" flag.
pub const DBI_START_MINIMIZED: u32 = 0x07;
/// `dbiSoundFlashBounceNotifyOld` — the legacy sound and flash-bounce mask.
pub const DBI_SOUND_FLASH_BOUNCE_NOTIFY_OLD: u32 = 0x08;
/// `dbiWorkModeOld` — the legacy tray/window work mode.
pub const DBI_WORK_MODE_OLD: u32 = 0x09;
/// `dbiSeenTrayTooltip` — the current "tray tooltip seen" flag.
pub const DBI_SEEN_TRAY_TOOLTIP: u32 = 0x0a;
/// `dbiDesktopNotifyOld` — the legacy desktop notifications flag.
pub const DBI_DESKTOP_NOTIFY_OLD: u32 = 0x0b;
/// `dbiAutoUpdate` — the current autoupdate flag.
pub const DBI_AUTO_UPDATE: u32 = 0x0c;
/// `dbiLastUpdateCheck` — the current last-update-check timestamp.
pub const DBI_LAST_UPDATE_CHECK: u32 = 0x0d;
/// `dbiWindowPositionOld` — the legacy window geometry.
pub const DBI_WINDOW_POSITION_OLD: u32 = 0x0e;
/// `dbiConnectionTypeOldOld` — the oldest connection type block.
pub const DBI_CONNECTION_TYPE_OLD_OLD: u32 = 0x0f;
// 0x10 reserved.
/// `dbiDefaultAttach` — the legacy default attach choice.
pub const DBI_DEFAULT_ATTACH: u32 = 0x11;
/// `dbiCatsAndDogsOld` — deprecated.
pub const DBI_CATS_AND_DOGS_OLD: u32 = 0x12;
/// `dbiReplaceEmojiOld` — the legacy "replace emoji" flag.
pub const DBI_REPLACE_EMOJI_OLD: u32 = 0x13;
/// `dbiAskDownloadPathOld` — the legacy "ask download path" flag.
pub const DBI_ASK_DOWNLOAD_PATH_OLD: u32 = 0x14;
/// `dbiDownloadPathOldOld` — the oldest download path block.
pub const DBI_DOWNLOAD_PATH_OLD_OLD: u32 = 0x15;
/// `dbiScaleOld` — the legacy enumerated interface scale.
pub const DBI_SCALE_OLD: u32 = 0x16;
/// `dbiEmojiTabOld` — deprecated.
pub const DBI_EMOJI_TAB_OLD: u32 = 0x17;
/// `dbiRecentEmojiOldOldOld` — the oldest recent-emoji block.
pub const DBI_RECENT_EMOJI_OLD_OLD_OLD: u32 = 0x18;
/// `dbiLoggedPhoneNumberOld` — deprecated.
pub const DBI_LOGGED_PHONE_NUMBER_OLD: u32 = 0x19;
/// `dbiMutedPeersOld` — deprecated.
pub const DBI_MUTED_PEERS_OLD: u32 = 0x1a;
// 0x1b reserved.
/// `dbiNotifyViewOld` — the legacy notification view mode.
pub const DBI_NOTIFY_VIEW_OLD: u32 = 0x1c;
/// `dbiSendToMenu` — the current "send to menu" flag.
pub const DBI_SEND_TO_MENU: u32 = 0x1d;
/// `dbiCompressPastedImageOld` — the legacy paste compression choice.
pub const DBI_COMPRESS_PASTED_IMAGE_OLD: u32 = 0x1e;
/// `dbiLangOld` — deprecated.
pub const DBI_LANG_OLD: u32 = 0x1f;
/// `dbiLangFileOld` — deprecated.
pub const DBI_LANG_FILE_OLD: u32 = 0x20;
/// `dbiTileBackgroundOld` — the legacy single tile flag.
pub const DBI_TILE_BACKGROUND_OLD: u32 = 0x21;
/// `dbiAutoLockOld` — the legacy auto-lock interval.
pub const DBI_AUTO_LOCK_OLD: u32 = 0x22;
/// `dbiDialogLastPath` — the current last dialog path.
pub const DBI_DIALOG_LAST_PATH: u32 = 0x23;
/// `dbiRecentEmojiOldOld` — recent emoji with 64-bit keys.
pub const DBI_RECENT_EMOJI_OLD_OLD: u32 = 0x24;
/// `dbiEmojiVariantsOldOld` — emoji skin variants with 32-bit keys.
pub const DBI_EMOJI_VARIANTS_OLD_OLD: u32 = 0x25;
/// `dbiRecentStickers` — the current recent stickers preload.
pub const DBI_RECENT_STICKERS: u32 = 0x26;
/// `dbiDcOptionOld` — a legacy dc endpoint with flags.
pub const DBI_DC_OPTION_OLD: u32 = 0x27;
/// `dbiTryIPv6Old` — the legacy "try IPv6" flag.
pub const DBI_TRY_IPV6_OLD: u32 = 0x28;
/// `dbiSongVolumeOld` — the legacy song volume.
pub const DBI_SONG_VOLUME_OLD: u32 = 0x29;
/// `dbiWindowsNotificationsOld` — a legacy notifications flag.
pub const DBI_WINDOWS_NOTIFICATIONS_OLD: u32 = 0x30;
/// `dbiIncludeMutedOld` — the legacy "include muted" flag.
pub const DBI_INCLUDE_MUTED_OLD: u32 = 0x31;
/// `dbiMegagroupSizeMaxOld` — a legacy fallback config limit.
pub const DBI_MEGAGROUP_SIZE_MAX_OLD: u32 = 0x32;
/// `dbiDownloadPathOld` — the download path with its bookmark.
pub const DBI_DOWNLOAD_PATH_OLD: u32 = 0x33;
/// `dbiAutoDownloadOld` — the legacy auto-download mask.
pub const DBI_AUTO_DOWNLOAD_OLD: u32 = 0x34;
/// `dbiSavedGifsLimitOld` — a legacy fallback config limit.
pub const DBI_SAVED_GIFS_LIMIT_OLD: u32 = 0x35;
/// `dbiShowingSavedGifsOld` — a legacy flag.
pub const DBI_SHOWING_SAVED_GIFS_OLD: u32 = 0x36;
/// `dbiAutoPlayOld` — the legacy single autoplay flag.
pub const DBI_AUTO_PLAY_OLD: u32 = 0x37;
/// `dbiAdaptiveForWideOld` — the legacy adaptive-for-wide flag.
pub const DBI_ADAPTIVE_FOR_WIDE_OLD: u32 = 0x38;
/// `dbiHiddenPinnedMessagesOld` — hidden pinned message ids per peer.
pub const DBI_HIDDEN_PINNED_MESSAGES_OLD: u32 = 0x39;
/// `dbiRecentEmojiOld` — recent emoji as string ids.
pub const DBI_RECENT_EMOJI_OLD: u32 = 0x3a;
/// `dbiEmojiVariantsOld` — emoji variants as string ids.
pub const DBI_EMOJI_VARIANTS_OLD: u32 = 0x3b;
/// `dbiDialogsModeOld` — the legacy dialogs mode.
pub const DBI_DIALOGS_MODE_OLD: u32 = 0x40;
/// `dbiModerateModeOld` — the legacy moderate mode flag.
pub const DBI_MODERATE_MODE_OLD: u32 = 0x41;
/// `dbiVideoVolumeOld` — the legacy video volume.
pub const DBI_VIDEO_VOLUME_OLD: u32 = 0x42;
/// `dbiStickersRecentLimitOld` — a legacy fallback config limit.
pub const DBI_STICKERS_RECENT_LIMIT_OLD: u32 = 0x43;
/// `dbiNativeNotificationsOld` — the legacy native notifications flag.
pub const DBI_NATIVE_NOTIFICATIONS_OLD: u32 = 0x44;
/// `dbiNotificationsCountOld` — the legacy notifications count.
pub const DBI_NOTIFICATIONS_COUNT_OLD: u32 = 0x45;
/// `dbiNotificationsCornerOld` — the legacy notifications corner.
pub const DBI_NOTIFICATIONS_CORNER_OLD: u32 = 0x46;
/// `dbiThemeKeyOld` — the legacy single theme key.
pub const DBI_THEME_KEY_OLD: u32 = 0x47;
/// `dbiDialogsWidthRatioOld` — the legacy dialogs width ratio.
pub const DBI_DIALOGS_WIDTH_RATIO_OLD: u32 = 0x48;
/// `dbiUseExternalVideoPlayerOld` — a legacy flag.
pub const DBI_USE_EXTERNAL_VIDEO_PLAYER_OLD: u32 = 0x49;
/// `dbiDcOptionsOld` — a serialized dc options blob.
pub const DBI_DC_OPTIONS_OLD: u32 = 0x4a;
/// `dbiMtpAuthorization` — the current serialized MTP authorization.
pub const DBI_MTP_AUTHORIZATION: u32 = 0x4b;
/// `dbiLastSeenWarningSeenOld` — the legacy last-seen warning flag.
pub const DBI_LAST_SEEN_WARNING_SEEN_OLD: u32 = 0x4c;
/// `dbiSessionSettings` — the serialized per-session settings.
pub const DBI_SESSION_SETTINGS: u32 = 0x4d;
/// `dbiLangPackKey` — the current language pack file key.
pub const DBI_LANG_PACK_KEY: u32 = 0x4e;
/// `dbiConnectionTypeOld` — the legacy proxy list block.
pub const DBI_CONNECTION_TYPE_OLD: u32 = 0x4f;
/// `dbiStickersFavedLimitOld` — a legacy fallback config limit.
pub const DBI_STICKERS_FAVED_LIMIT_OLD: u32 = 0x50;
/// `dbiSuggestStickersByEmojiOld` — the legacy suggest-stickers flag.
pub const DBI_SUGGEST_STICKERS_BY_EMOJI_OLD: u32 = 0x51;
/// `dbiSuggestEmojiOld` — the legacy suggest-emoji flag.
pub const DBI_SUGGEST_EMOJI_OLD: u32 = 0x52;
/// `dbiTxtDomainStringOldOld` — the oldest txt domain block.
pub const DBI_TXT_DOMAIN_STRING_OLD_OLD: u32 = 0x53;
/// `dbiThemeKey` — the current day and night theme keys.
pub const DBI_THEME_KEY: u32 = 0x54;
/// `dbiTileBackground` — the current tile flags for both themes.
pub const DBI_TILE_BACKGROUND: u32 = 0x55;
/// `dbiCacheSettingsOld` — the legacy single cache limit pair.
pub const DBI_CACHE_SETTINGS_OLD: u32 = 0x56;
/// `dbiPowerSaving` — the current power saving flags.
pub const DBI_POWER_SAVING: u32 = 0x57;
/// `dbiScalePercent` — the current interface scale percentage.
pub const DBI_SCALE_PERCENT: u32 = 0x58;
/// `dbiPlaybackSpeedOld` — the legacy 2x playback flag.
pub const DBI_PLAYBACK_SPEED_OLD: u32 = 0x59;
/// `dbiLanguagesKey` — the current languages file key.
pub const DBI_LANGUAGES_KEY: u32 = 0x5a;
/// `dbiCallSettingsOld` — the legacy call devices and volumes.
pub const DBI_CALL_SETTINGS_OLD: u32 = 0x5b;
/// `dbiCacheSettings` — the current cache limit pairs.
pub const DBI_CACHE_SETTINGS: u32 = 0x5c;
/// `dbiTxtDomainStringOld` — the legacy txt domain string.
pub const DBI_TXT_DOMAIN_STRING_OLD: u32 = 0x5d;
/// `dbiApplicationSettings` — the serialized application settings.
pub const DBI_APPLICATION_SETTINGS: u32 = 0x5e;
/// `dbiDialogsFiltersOld` — the legacy dialogs filters flag.
pub const DBI_DIALOGS_FILTERS_OLD: u32 = 0x5f;
/// `dbiFallbackProductionConfig` — the serialized fallback config.
pub const DBI_FALLBACK_PRODUCTION_CONFIG: u32 = 0x60;
/// `dbiBackgroundKey` — the day and night background file keys.
pub const DBI_BACKGROUND_KEY: u32 = 0x61;

/// `dbiEncryptedWithSalt` — a marker block of the oldest format.
pub const DBI_ENCRYPTED_WITH_SALT: u32 = 333;
/// `dbiEncrypted` — a marker block of the oldest format.
pub const DBI_ENCRYPTED: u32 = 444;

/// `dbiVersion` — the trailing version block.
pub const DBI_VERSION: u32 = 666;

/// Whether `ReadSetting` handles this id; anything else fails the whole read.
///
/// The three marker blocks of the oldest format — [`DBI_ENCRYPTED_WITH_SALT`],
/// [`DBI_ENCRYPTED`] and [`DBI_VERSION`] — are handled by the old-format reader
/// in `storage/localstorage.cpp`, never by the settings scheme, so they are
/// unknown here exactly as they are in C++.
#[must_use]
pub fn is_known(id: u32) -> bool {
    KNOWN.contains(&id)
}

/// Every id `ReadSetting` switches on, for the unknown-block check.
pub const KNOWN: [u32; 86] = [
    DBI_KEY,
    DBI_USER,
    DBI_DC_OPTION_OLD_OLD,
    DBI_CHAT_SIZE_MAX_OLD,
    DBI_MUTE_PEER_OLD,
    DBI_SEND_KEY_OLD,
    DBI_AUTO_START,
    DBI_START_MINIMIZED,
    DBI_SOUND_FLASH_BOUNCE_NOTIFY_OLD,
    DBI_WORK_MODE_OLD,
    DBI_SEEN_TRAY_TOOLTIP,
    DBI_DESKTOP_NOTIFY_OLD,
    DBI_AUTO_UPDATE,
    DBI_LAST_UPDATE_CHECK,
    DBI_WINDOW_POSITION_OLD,
    DBI_CONNECTION_TYPE_OLD_OLD,
    DBI_DEFAULT_ATTACH,
    DBI_CATS_AND_DOGS_OLD,
    DBI_REPLACE_EMOJI_OLD,
    DBI_ASK_DOWNLOAD_PATH_OLD,
    DBI_DOWNLOAD_PATH_OLD_OLD,
    DBI_SCALE_OLD,
    DBI_EMOJI_TAB_OLD,
    DBI_RECENT_EMOJI_OLD_OLD_OLD,
    DBI_LOGGED_PHONE_NUMBER_OLD,
    DBI_MUTED_PEERS_OLD,
    DBI_NOTIFY_VIEW_OLD,
    DBI_SEND_TO_MENU,
    DBI_COMPRESS_PASTED_IMAGE_OLD,
    DBI_LANG_OLD,
    DBI_LANG_FILE_OLD,
    DBI_TILE_BACKGROUND_OLD,
    DBI_AUTO_LOCK_OLD,
    DBI_DIALOG_LAST_PATH,
    DBI_RECENT_EMOJI_OLD_OLD,
    DBI_EMOJI_VARIANTS_OLD_OLD,
    DBI_RECENT_STICKERS,
    DBI_DC_OPTION_OLD,
    DBI_TRY_IPV6_OLD,
    DBI_SONG_VOLUME_OLD,
    DBI_WINDOWS_NOTIFICATIONS_OLD,
    DBI_INCLUDE_MUTED_OLD,
    DBI_MEGAGROUP_SIZE_MAX_OLD,
    DBI_DOWNLOAD_PATH_OLD,
    DBI_AUTO_DOWNLOAD_OLD,
    DBI_SAVED_GIFS_LIMIT_OLD,
    DBI_SHOWING_SAVED_GIFS_OLD,
    DBI_AUTO_PLAY_OLD,
    DBI_ADAPTIVE_FOR_WIDE_OLD,
    DBI_HIDDEN_PINNED_MESSAGES_OLD,
    DBI_RECENT_EMOJI_OLD,
    DBI_EMOJI_VARIANTS_OLD,
    DBI_DIALOGS_MODE_OLD,
    DBI_MODERATE_MODE_OLD,
    DBI_VIDEO_VOLUME_OLD,
    DBI_STICKERS_RECENT_LIMIT_OLD,
    DBI_NATIVE_NOTIFICATIONS_OLD,
    DBI_NOTIFICATIONS_COUNT_OLD,
    DBI_NOTIFICATIONS_CORNER_OLD,
    DBI_THEME_KEY_OLD,
    DBI_DIALOGS_WIDTH_RATIO_OLD,
    DBI_USE_EXTERNAL_VIDEO_PLAYER_OLD,
    DBI_DC_OPTIONS_OLD,
    DBI_MTP_AUTHORIZATION,
    DBI_LAST_SEEN_WARNING_SEEN_OLD,
    DBI_SESSION_SETTINGS,
    DBI_LANG_PACK_KEY,
    DBI_CONNECTION_TYPE_OLD,
    DBI_STICKERS_FAVED_LIMIT_OLD,
    DBI_SUGGEST_STICKERS_BY_EMOJI_OLD,
    DBI_SUGGEST_EMOJI_OLD,
    DBI_TXT_DOMAIN_STRING_OLD_OLD,
    DBI_THEME_KEY,
    DBI_TILE_BACKGROUND,
    DBI_CACHE_SETTINGS_OLD,
    DBI_POWER_SAVING,
    DBI_SCALE_PERCENT,
    DBI_PLAYBACK_SPEED_OLD,
    DBI_LANGUAGES_KEY,
    DBI_CALL_SETTINGS_OLD,
    DBI_CACHE_SETTINGS,
    DBI_TXT_DOMAIN_STRING_OLD,
    DBI_APPLICATION_SETTINGS,
    DBI_DIALOGS_FILTERS_OLD,
    DBI_FALLBACK_PRODUCTION_CONFIG,
    DBI_BACKGROUND_KEY,
];