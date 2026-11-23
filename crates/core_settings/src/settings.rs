//! Application Settings ported from Telegram Desktop.
//!
//! Source: `tdesktop/Telegram/SourceFiles/core/core_settings.{h,cpp}`.

use crate::types::*;
use crate::window_position::WindowPosition;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    pub adaptive_for_wide: bool,
    pub moderate_mode_enabled: bool,
    pub song_volume: f64,
    pub video_volume: f64,
    pub ask_download_path: bool,
    pub download_path: String,
    pub download_path_bookmark: Vec<u8>,
    pub sound_notify: bool,
    pub desktop_notify: bool,
    pub flash_bounce_notify: bool,
    pub notify_view: NotifyView,
    pub native_notifications: bool,
    pub notifications_count: i32,
    pub notifications_corner: ScreenCorner,
    pub auto_lock: i32,
    pub last_seen_warning_seen: bool,
    pub send_submit_way: InputSubmitSettings,
    pub include_muted_counter: bool,
    pub count_unread_messages: bool,
    pub notify_about_pinned: bool,
    pub loop_animated_stickers: bool,
    pub large_emoji: bool,
    pub replace_emoji: bool,
    pub suggest_emoji: bool,
    pub suggest_stickers_by_emoji: bool,
    pub spellchecker_enabled: bool,
    pub tabbed_selector_section_enabled: bool,
    pub work_mode: WorkMode,
    pub window_position: WindowPosition,
    pub window_columns: WindowColumns,
    pub window_title_content: WindowTitleContent,
    pub sound_overrides: HashMap<String, String>,
    pub accounts_order: Vec<u64>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            adaptive_for_wide: true,
            moderate_mode_enabled: false,
            song_volume: 0.8,
            video_volume: 0.8,
            ask_download_path: false,
            download_path: String::new(),
            download_path_bookmark: Vec::new(),
            sound_notify: true,
            desktop_notify: true,
            flash_bounce_notify: true,
            notify_view: NotifyView::ShowPreview,
            native_notifications: false,
            notifications_count: 3,
            notifications_corner: ScreenCorner::BottomRight,
            auto_lock: 3600,
            last_seen_warning_seen: false,
            send_submit_way: InputSubmitSettings::Enter,
            include_muted_counter: true,
            count_unread_messages: false,
            notify_about_pinned: true,
            loop_animated_stickers: true,
            large_emoji: true,
            replace_emoji: true,
            suggest_emoji: true,
            suggest_stickers_by_emoji: true,
            spellchecker_enabled: true,
            tabbed_selector_section_enabled: false,
            work_mode: WorkMode::WindowAndTray,
            window_position: WindowPosition::default(),
            window_columns: WindowColumns::default(),
            window_title_content: WindowTitleContent::default(),
            sound_overrides: HashMap::new(),
            accounts_order: Vec::new(),
        }
    }
}

impl Settings {
    pub fn new() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_settings_default() {
        let s = Settings::new();
        assert!(s.adaptive_for_wide);
        assert!(s.sound_notify);
        assert_eq!(s.work_mode, WorkMode::WindowAndTray);
        assert_eq!(s.notifications_corner, ScreenCorner::BottomRight);
    }
}
