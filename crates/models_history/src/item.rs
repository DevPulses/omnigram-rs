//! HistoryItem and History structures ported from Telegram Desktop.
//!
//! Source: `tdesktop/Telegram/SourceFiles/history/history_item.{h,cpp}`.

use crate::flags::{MessageFlags, MsgId};
use models_peers::{PeerId, UserId};

/// Information about a reply to another message or topic.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplyToInfo {
    pub reply_to_msg_id: MsgId,
    pub reply_to_top_id: MsgId,
    pub reply_to_peer_id: Option<PeerId>,
}

/// Message markup button definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineButton {
    pub text: String,
    pub data: Vec<u8>,
    pub forward_text: String,
}

/// A row of inline buttons.
pub type InlineButtonRow = Vec<InlineButton>;

/// Representation of a Telegram message or service action in history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryItem {
    pub id: MsgId,
    pub peer_id: PeerId,
    pub from: PeerId,
    pub date: i32,
    pub flags: MessageFlags,
    pub text: String,
    pub reply_to: Option<ReplyToInfo>,
    pub post_author: String,
    pub via_bot_id: UserId,
    pub grouped_id: u64,
    pub views: i32,
    pub forwards: i32,
    pub edit_date: i32,
}

impl HistoryItem {
    pub fn new(id: MsgId, peer_id: PeerId, date: i32) -> Self {
        Self {
            id,
            peer_id,
            from: PeerId::ZERO,
            date,
            flags: MessageFlags::empty(),
            text: String::new(),
            reply_to: None,
            post_author: String::new(),
            via_bot_id: UserId::new(0),
            grouped_id: 0,
            views: -1,
            forwards: -1,
            edit_date: 0,
        }
    }

    pub fn is_outgoing(&self) -> bool {
        self.flags.contains(MessageFlags::Outgoing)
    }

    pub fn is_unread(&self) -> bool {
        self.flags.contains(MessageFlags::ClientSideUnread)
            || (self.flags.contains(MessageFlags::Outgoing)
                && !self.flags.contains(MessageFlags::HistoryEntry))
    }

    pub fn is_silent(&self) -> bool {
        self.flags.contains(MessageFlags::Silent)
    }

    pub fn is_pinned(&self) -> bool {
        self.flags.contains(MessageFlags::Pinned)
    }

    pub fn is_post(&self) -> bool {
        self.flags.contains(MessageFlags::Post)
    }

    pub fn is_edited(&self) -> bool {
        self.edit_date > 0 && !self.flags.contains(MessageFlags::HideEdited)
    }
}

/// Per-peer chat message history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct History {
    pub peer_id: PeerId,
    pub unread_count: i32,
    pub inbox_read_before: MsgId,
    pub outbox_read_before: MsgId,
    pub items: Vec<HistoryItem>,
}

impl History {
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            unread_count: 0,
            inbox_read_before: 0,
            outbox_read_before: 0,
            items: Vec::new(),
        }
    }

    pub fn add_item(&mut self, item: HistoryItem) {
        if let Some(pos) = self.items.iter().position(|it| it.id == item.id) {
            self.items[pos] = item;
        } else {
            self.items.push(item);
            self.items.sort_by_key(|it| it.id);
        }
    }

    pub fn get_item(&self, id: MsgId) -> Option<&HistoryItem> {
        self.items.iter().find(|it| it.id == id)
    }

    pub fn remove_item(&mut self, id: MsgId) -> Option<HistoryItem> {
        if let Some(pos) = self.items.iter().position(|it| it.id == id) {
            Some(self.items.remove(pos))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_peers::UserId;

    #[test]
    fn test_history_item_flags() {
        let p = PeerId::from_user(UserId::new(100));
        let mut item = HistoryItem::new(1, p, 1600000000);
        item.flags.insert(MessageFlags::Outgoing);
        item.flags.insert(MessageFlags::Silent);
        assert!(item.is_outgoing());
        assert!(item.is_silent());
        assert!(!item.is_pinned());
    }

    #[test]
    fn test_history_operations() {
        let p = PeerId::from_user(UserId::new(100));
        let mut h = History::new(p);

        let item2 = HistoryItem::new(2, p, 1600000002);
        let item1 = HistoryItem::new(1, p, 1600000001);
        h.add_item(item2);
        h.add_item(item1);

        assert_eq!(h.items.len(), 2);
        assert_eq!(h.items[0].id, 1);
        assert_eq!(h.items[1].id, 2);

        assert!(h.get_item(1).is_some());
        assert_eq!(h.remove_item(1).unwrap().id, 1);
        assert_eq!(h.items.len(), 1);
    }
}
