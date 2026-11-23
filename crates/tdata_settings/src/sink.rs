//! The effects a settings pass hands to the application.
//!
//! Most blocks of `ReadSetting` write into `Core::App().settings()`, the theme,
//! the proxy configuration or the platform autostart, all of which belong to
//! areas outside this task. The pass reports exactly the values it would have
//! written, in the same order, and the caller applies them.
//!
//! Values the C++ code cannot express without a further model — the emoji id
//! mapping, the peer id decoding, the autodownload byte limits — are reported
//! in the raw form the file carries, with the conversion noted on the variant.

/// `MTP::ProxyData::Type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyType {
    None,
    Socks5,
    Http,
    Mtproto,
    Web,
}

/// `MTP::ProxyData::Settings`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxySettings {
    System,
    Enabled,
    Disabled,
}

/// One proxy entry as the legacy blocks store it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyData {
    pub kind: Option<ProxyType>,
    pub host: String,
    pub port: u32,
    pub user: String,
    pub password: String,
}

impl ProxyData {
    /// `operator bool` — a proxy is usable once it has a type and a host.
    #[must_use]
    pub fn is_set(&self) -> bool {
        !matches!(self.kind, None | Some(ProxyType::None)) && !self.host.is_empty()
    }
}

/// `Core::WindowPosition`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub moncrc: u64,
    pub maximized: bool,
}

/// `Core::Settings::CallSettings` as the legacy block stores it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CallSettings {
    pub output_device_id: String,
    pub output_volume: i32,
    pub input_device_id: String,
    pub input_volume: i32,
    pub ducking_enabled: i32,
}

/// One value the C++ code would have written while reading a block.
#[derive(Debug, Clone, PartialEq)]
pub enum LegacyEffect {
    /// `Core::App().settings().addFromSerialized(serialized, version)`.
    ApplicationSettings { serialized: Vec<u8>, version: i32 },
    AutoStart(bool),
    StartMinimized(bool),
    SendToMenu(bool),
    SeenTrayTooltip(bool),
    AutoUpdate(bool),
    /// `cSetLastUpdateCheck` — seconds since the epoch.
    LastUpdateCheck(i32),
    /// `cSetDialogLastPath`.
    DialogLastPath(String),
    DesktopNotify(bool),
    NativeNotifications(bool),
    /// Already normalised: a stored value `<= 0` becomes 3.
    NotificationsCount(i32),
    /// Already normalised to `0..=3`, anything else becomes `2`
    /// (`ScreenCorner::BottomRight`).
    NotificationsCorner(i32),
    IncludeMutedCounter(bool),
    ModerateModeEnabled(bool),
    /// From `dbiSoundFlashBounceNotifyOld`, which sets both in one block.
    SoundNotify(bool),
    FlashBounceNotify(bool),
    /// `Core::Settings::WorkMode` — `0` WindowAndTray, `1` TrayOnly,
    /// `2` WindowOnly, anything else WindowAndTray.
    WorkMode(i32),
    /// `Core::Settings::NotifyView` — `0` ShowPreview, `1` ShowName,
    /// `2` ShowNothing, anything else ShowPreview.
    NotifyView(i32),
    /// `updateDialogsWidthRatio(value / 1000000.)`.
    DialogsWidthRatio(f64),
    LastSeenWarningSeen(bool),
    WindowPosition(WindowPosition),
    /// `Ui::InputSubmitSettings` — `0` Enter, `1` Ctrl+Enter.
    SendSubmitWay(i32),
    AdaptiveForWide(bool),
    /// Auto-lock interval in seconds.
    AutoLock(i32),
    ReplaceEmoji(bool),
    SuggestEmoji(bool),
    SuggestStickersByEmoji(bool),
    AskDownloadPath(bool),
    DownloadPath {
        path: String,
        bookmark: Vec<u8>,
    },
    /// `Ui::SendFilesWay` with group files and send-as-photos set together.
    SendFilesWay { group_files: bool },
    /// `PowerSaving::Flags::from_raw`.
    PowerSaving(i32),
    /// `SetScaleChecked` — `0` is automatic, otherwise a percentage.
    Scale(i32),
    /// One write of the proxy configuration: the list, the selected entry, the
    /// mode, and — for the newer legacy block — the calls and IPv6 flags.
    ProxySelection {
        list: Vec<ProxyData>,
        selected: Option<ProxyData>,
        settings: ProxySettings,
        use_for_calls: Option<bool>,
        try_ipv6: Option<bool>,
    },
    /// `proxy().setTryIPv6` from `dbiTryIPv6Old`.
    ProxyTryIPv6(bool),
    /// `Window::Theme::SetNightModeValue`.
    ThemeNightMode(bool),
    /// `dbiRecentStickers` — `RecentStickerPreload`, pairs of document id and
    /// use count, set through `cSetRecentStickersPreload`.
    RecentStickersPreload(Vec<(u64, u16)>),
    /// Legacy recent emoji, still in old-key form: the emoji id mapping lives
    /// in the emoji area, so the sink converts.
    LegacyRecentEmoji(Vec<(u64, u16)>),
    /// Legacy recent emoji already stored as string ids.
    LegacyRecentEmojiById(Vec<(String, u16)>),
    /// Legacy emoji variants: old emoji key to old colour key.
    LegacyEmojiVariants(Vec<(u64, u64)>),
    /// Legacy emoji variants already stored as string ids.
    LegacyEmojiVariantsById(Vec<(String, i32)>),
    /// `setSongVolume(clamp(value / 1e6, 0, 1))`.
    SongVolume(f64),
    /// `setVideoVolume(clamp(value / 1e6, 0, 1))`.
    VideoVolume(f64),
    /// `setVoicePlaybackSpeed(v)` and `setAudioPlaybackSpeed(v)`.
    PlaybackSpeed(f64),
    CallSettings(CallSettings),
    /// `dbiAutoDownloadOld` masks, before they become autodownload byte
    /// limits: bit `0x01` disables private chats, bit `0x02` groups and
    /// channels.
    AutoDownloadMasks { photo: i32, audio: i32, gif: i32 },
    /// `dbiAutoPlayOld` — `false` zeroes the autoplay byte limits.
    AutoPlay(bool),
}

/// Where a settings pass delivers its effects.
pub trait SettingsSink {
    /// Applies one effect. Called in file order, once per value.
    fn apply(&mut self, effect: LegacyEffect);
}

/// A sink that keeps every effect, for tests and for callers that want to
/// replay a pass later.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct RecordingSink {
    pub effects: Vec<LegacyEffect>,
}

impl RecordingSink {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl SettingsSink for RecordingSink {
    fn apply(&mut self, effect: LegacyEffect) {
        self.effects.push(effect);
    }
}