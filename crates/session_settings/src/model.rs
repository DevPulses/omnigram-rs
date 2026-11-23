//! Per-session settings and serialisation.
//!
//! Port of `tdesktop/Telegram/SourceFiles/main/main_session_settings.{h,cpp}`.

use models_peers::PeerId;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectorTab {
    #[default]
    Emoji = 0,
    Stickers = 1,
    Gifs = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SupportSwitchSettings {
    #[default]
    Next = 0,
    Previous = 1,
    None = 2,
}

/// Settings specific to a single user session/account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSettings {
    pub selector_tab: SelectorTab,
    pub support_switch: SupportSwitchSettings,
    pub support_fix_chats_order: bool,
    pub support_templates_autocomplete: bool,
    pub support_chats_time_slice: i32,
    pub support_all_search_results: bool,
    pub support_all_silent: bool,
    pub archive_collapsed: bool,
    pub archive_in_main_menu: bool,
    pub dialogs_filters_enabled: bool,
    pub group_stickers_section_hidden: HashSet<PeerId>,
    pub group_emoji_section_hidden: HashSet<PeerId>,
    pub hidden_pinned_messages: Vec<(u64, i32)>,
    pub mute_periods: Vec<u64>,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            selector_tab: SelectorTab::Emoji,
            support_switch: SupportSwitchSettings::Next,
            support_fix_chats_order: false,
            support_templates_autocomplete: true,
            support_chats_time_slice: 0,
            support_all_search_results: false,
            support_all_silent: false,
            archive_collapsed: false,
            archive_in_main_menu: false,
            dialogs_filters_enabled: true,
            group_stickers_section_hidden: HashSet::new(),
            group_emoji_section_hidden: HashSet::new(),
            hidden_pinned_messages: Vec::new(),
            mute_periods: Vec::new(),
        }
    }
}

impl SessionSettings {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_group_stickers_section_hidden(&self, peer_id: PeerId) -> bool {
        self.group_stickers_section_hidden.contains(&peer_id)
    }

    pub fn set_group_stickers_section_hidden(&mut self, peer_id: PeerId) {
        self.group_stickers_section_hidden.insert(peer_id);
    }

    pub fn is_group_emoji_section_hidden(&self, peer_id: PeerId) -> bool {
        self.group_emoji_section_hidden.contains(&peer_id)
    }

    pub fn set_group_emoji_section_hidden(&mut self, peer_id: PeerId) {
        self.group_emoji_section_hidden.insert(peer_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_peers::UserId;

    #[test]
    fn test_session_settings_default() {
        let s = SessionSettings::new();
        assert_eq!(s.selector_tab, SelectorTab::Emoji);
        assert!(s.dialogs_filters_enabled);
        assert!(!s.archive_collapsed);
    }

    #[test]
    fn test_group_stickers_hidden() {
        let mut s = SessionSettings::new();
        let p = PeerId::from_user(UserId::new(1234));
        assert!(!s.is_group_stickers_section_hidden(p));
        s.set_group_stickers_section_hidden(p);
        assert!(s.is_group_stickers_section_hidden(p));
    }
}
