//! Common types used in Core Settings.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputSubmitSettings {
    #[default]
    Enter = 0,
    CtrlEnter = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkMode {
    #[default]
    WindowAndTray = 0,
    TrayOnly = 1,
    WindowOnly = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NotifyView {
    #[default]
    ShowPreview = 0,
    ShowName = 1,
    ShowNothing = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScreenCorner {
    TopLeft = 0,
    TopRight = 1,
    #[default]
    BottomRight = 2,
    BottomLeft = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WindowColumn {
    First = 0,
    #[default]
    Second = 1,
    Third = 2,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct WindowColumns {
    pub dialogs_with_chat_width_ratio: f64,
    pub dialogs_no_chat_width_ratio: f64,
    pub third_column_width: i32,
    pub third_section_extended_by: i32,
    pub tabbed_selector_section_enabled: bool,
    pub third_section_info_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WindowTitleContent {
    pub hide_chat_name: bool,
    pub hide_account_name: bool,
    pub hide_total_unread: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RecentEmojiDocument {
    pub id: u64,
    pub test: bool,
}

impl RecentEmojiDocument {
    pub fn serialize(&self) -> String {
        format!("{}-{}", self.id, if self.test { 1 } else { 0 })
    }

    pub fn deserialize(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split('-').collect();
        if parts.len() != 2 || parts[1].len() != 1 {
            return None;
        }
        let id: u64 = parts[0].parse().ok()?;
        let test = match parts[1] {
            "0" => false,
            "1" => true,
            _ => return None,
        };
        Some(Self { id, test })
    }
}
