//! Common dialogs types.
//!
//! Ported from `dialogs/dialogs_common.h`.

use models_history::MsgId;

/// Sort mode for dialog lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortMode {
    /// Default chat list order (date/pinned/fixed ordering).
    #[default]
    Date,
    /// `Date` + `name` threshold for add-by-name insertion.
    Add,
    /// Strict name sorting.
    Name,
}

/// Unread counters/state for an entry.
///
/// Mirrors the fields used by `Dialogs::UnreadState` in tdesktop.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UnreadState {
    pub messages: i32,
    pub chats: i32,
    pub marks: i32,
    pub reactions: i32,
    pub mentions: i32,
    pub messages_muted: bool,
    pub chats_muted: bool,
    pub marks_muted: bool,
    pub reactions_muted: bool,
    pub known: Option<bool>,
}

impl UnreadState {
    pub const fn empty() -> Self {
        Self {
            messages: 0,
            chats: 0,
            marks: 0,
            reactions: 0,
            mentions: 0,
            messages_muted: false,
            chats_muted: false,
            marks_muted: false,
            reactions_muted: false,
            known: None,
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.messages == 0
            && self.chats == 0
            && self.marks == 0
            && self.reactions == 0
            && self.mentions == 0
    }

    pub fn add(&mut self, other: UnreadState) {
        self.messages += other.messages;
        self.chats += other.chats;
        self.marks += other.marks;
        self.reactions += other.reactions;
        self.mentions += other.mentions;
        self.messages_muted |= other.messages_muted;
        self.chats_muted |= other.chats_muted;
        self.marks_muted |= other.marks_muted;
        self.reactions_muted |= other.reactions_muted;
        self.known = match (self.known, other.known) {
            (Some(a), Some(b)) => Some(a && b),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
    }
}

/// Cached date label for chat list rows (`Dialogs::DateTextCache`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DateTextCache {
    pub date_text: String,
    pub text_date: String,
    pub item_date: i32,
    pub item_msg_id: MsgId,
    pub day_id: i32,
}

impl DateTextCache {
    pub fn clear(&mut self) {
        self.date_text.clear();
        self.text_date.clear();
        self.item_date = 0;
        self.item_msg_id = 0;
        self.day_id = 0;
    }
}
