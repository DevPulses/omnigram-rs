//! What the scheme read: the `ReadSettingsContext` of the C++ port.
//!
//! Source of truth: `ReadSettingsContext` in
//! `storage/details/storage_settings_scheme.h`.
//!
//! Every field here is a value the pass collects for someone else to apply:
//! the fallback config pieces, the cache limits, the theme and language file
//! keys, the MTP authorization and the legacy keys. Values that the C++ code
//! pushes straight into `Core::App().settings()` are not context — they leave
//! the pass as [`crate::sink::LegacyEffect`]s.

/// `MTP::AuthKey::Data` — the 256-byte key a legacy block carries.
pub const LEGACY_AUTH_KEY_SIZE: usize = 256;

/// One dc endpoint of a legacy block, before it becomes a `DcOptions` entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyDcEndpoint {
    pub dc_id: u32,
    pub flags: i32,
    pub ip: String,
    pub port: u32,
}

/// `fallbackConfigLegacyDcOptions` — a `DcOptions` built by the pass.
///
/// The C++ field is a live `MTP::DcOptions` object: the two `dbiDcOption*`
/// blocks add single endpoints to it and `dbiDcOptionsOld` replaces it from a
/// serialized blob. The owned store is `mtproto`'s area, so a serialized blob
/// is kept as bytes here and parsed where that store lives.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum LegacyDcOptions {
    /// Nothing was read.
    #[default]
    Empty,
    /// Endpoints added one by one by the oldest blocks.
    Endpoints(Vec<LegacyDcEndpoint>),
    /// A blob from `dbiDcOptionsOld`, not parsed here.
    Serialized(Vec<u8>),
}

/// A legacy MTP authorization key: `MTP::AuthKey::Type::ReadFromFile`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyAuthKey {
    pub dc_id: i32,
    pub key: [u8; LEGACY_AUTH_KEY_SIZE],
}

/// What the pass collected for `Main::SessionSettings`.
///
/// The model itself belongs to `main/main_session_settings`, which is outside
/// this task, so the pass keeps the serialized payload and the individual
/// values the legacy blocks set. Merging them into one object happens in the
/// model, in the same order the blocks were read.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionSettingsInput {
    /// `addFromSerialized` payload from `dbiSessionSettings`, with its version.
    pub serialized: Option<(Vec<u8>, i32)>,
    /// `setDialogsFiltersEnabled` from `dbiDialogsFiltersOld`.
    pub dialogs_filters_enabled: Option<bool>,
    /// `setHiddenPinnedMessageId(peer, MsgId(0), PeerId(0), MsgId(value))`.
    pub hidden_pinned_messages: Vec<(u64, i32)>,
}

/// `ReadSettingsContext` — everything one settings pass collects.
#[derive(Debug, Clone)]
pub struct ReadSettingsContext {
    /// Read by `dbiTileBackgroundOld`: an old tile flag is ignored when the
    /// account had a custom day background and the file predates 8005.
    pub legacy_has_custom_day_background: bool,

    pub fallback_config_legacy_dc_options: LegacyDcOptions,
    pub fallback_config_legacy_chat_size_max: i32,
    pub fallback_config_legacy_saved_gifs_limit: i32,
    pub fallback_config_legacy_stickers_recent_limit: i32,
    pub fallback_config_legacy_stickers_faved_limit: i32,
    pub fallback_config_legacy_megagroup_size_max: i32,
    pub fallback_config_legacy_txt_domain_string: String,
    /// `fallbackConfig` — a serialized production config, if the file has one.
    pub fallback_config: Vec<u8>,

    pub cache_total_size_limit: i64,
    pub cache_total_time_limit: i32,
    pub cache_big_file_total_size_limit: i64,
    pub cache_big_file_total_time_limit: i32,

    pub session_settings: SessionSettingsInput,

    pub theme_key_legacy: u64,
    pub theme_key_day: u64,
    pub theme_key_night: u64,
    pub background_key_day: u64,
    pub background_key_night: u64,
    pub background_keys_read: bool,
    pub tile_day: bool,
    pub tile_night: bool,
    pub tile_read: bool,
    pub lang_pack_key: u64,
    pub languages_key: u64,

    pub mtp_authorization: Vec<u8>,
    pub mtp_legacy_keys: Vec<LegacyAuthKey>,
    pub mtp_legacy_main_dc_id: i32,
    pub mtp_legacy_user_id: i32,

    /// Whether any legacy block was read, which makes the C++ code rewrite the
    /// settings file in the current format.
    pub legacy_read: bool,

    /// The night mode the pass itself established, standing in for
    /// `Window::Theme::IsNightMode()` while `dbiTileBackgroundOld` is read.
    pub night_mode: bool,

    /// The scale forced by the command line, standing in for `cConfigScale()`.
    /// `None` means "not set", which behaves like `style::kScaleAuto`.
    pub scale_from_command_line: Option<i32>,
}

impl Default for ReadSettingsContext {
    fn default() -> Self {
        Self {
            legacy_has_custom_day_background: false,
            fallback_config_legacy_dc_options: LegacyDcOptions::Empty,
            fallback_config_legacy_chat_size_max: 0,
            fallback_config_legacy_saved_gifs_limit: 0,
            fallback_config_legacy_stickers_recent_limit: 0,
            fallback_config_legacy_stickers_faved_limit: 0,
            fallback_config_legacy_megagroup_size_max: 0,
            fallback_config_legacy_txt_domain_string: String::new(),
            fallback_config: Vec::new(),
            cache_total_size_limit: 0,
            cache_total_time_limit: 0,
            cache_big_file_total_size_limit: 0,
            cache_big_file_total_time_limit: 0,
            session_settings: SessionSettingsInput::default(),
            theme_key_legacy: 0,
            theme_key_day: 0,
            theme_key_night: 0,
            background_key_day: 0,
            background_key_night: 0,
            background_keys_read: false,
            // The C++ defaults are tileDay = false and tileNight = true.
            tile_day: false,
            tile_night: true,
            tile_read: false,
            lang_pack_key: 0,
            languages_key: 0,
            mtp_authorization: Vec::new(),
            mtp_legacy_keys: Vec::new(),
            mtp_legacy_main_dc_id: 0,
            mtp_legacy_user_id: 0,
            legacy_read: false,
            night_mode: false,
            scale_from_command_line: None,
        }
    }
}

impl ReadSettingsContext {
    /// A context with no legacy state, like `Account::prepareReadSettingsContext`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The same context, told whether the account has a custom day background.
    #[must_use]
    pub fn with_legacy_day_background(mut self, has: bool) -> Self {
        self.legacy_has_custom_day_background = has;
        self
    }
}