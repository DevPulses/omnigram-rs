//! `ChatFilter` flags, inclusion/exclusion rules, and `ChatFiltersManager`.
//!
//! Source: `tdesktop/Telegram/SourceFiles/data/data_chat_filters.{h,cpp}`.

use std::collections::{HashMap, HashSet};

use models_peers::PeerId;

pub type FilterId = i32;
pub const MAIN_FILTER_ID: FilterId = 0;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct ChatFilterFlags: u16 {
        const CONTACTS     = 1 << 0;
        const NON_CONTACTS = 1 << 1;
        const GROUPS       = 1 << 2;
        const CHANNELS     = 1 << 3;
        const BOTS         = 1 << 4;
        const NO_MUTED     = 1 << 5;
        const NO_READ      = 1 << 6;
        const NO_ARCHIVED  = 1 << 7;
        const RULES_MASK   = (1 << 8) - 1;

        const CHATLIST     = 1 << 8;
        const HAS_MY_LINKS = 1 << 9;
        const STATIC_TITLE = 1 << 10;

        const NEW_CHATS      = 1 << 11;
        const EXISTING_CHATS = 1 << 12;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChatFilterTitle {
    pub text: String,
    pub is_static: bool,
}

impl ChatFilterTitle {
    pub fn new(text: impl Into<String>, is_static: bool) -> Self {
        Self {
            text: text.into(),
            is_static,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
}

/// Lightweight peer/dialog attributes evaluated against a `ChatFilter`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilterChatDescriptor {
    pub peer_id: PeerId,
    pub is_contact: bool,
    pub is_user: bool,
    pub is_group: bool,
    pub is_channel: bool,
    pub is_bot: bool,
    pub is_muted: bool,
    pub is_unread: bool,
    pub is_archived: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatFilter {
    pub id: FilterId,
    pub title: ChatFilterTitle,
    pub icon_emoji: String,
    pub color_index: Option<u8>,
    pub flags: ChatFilterFlags,
    pub always: HashSet<PeerId>,
    pub pinned: Vec<PeerId>,
    pub never: HashSet<PeerId>,
}

impl ChatFilter {
    pub fn new(
        id: FilterId,
        title: ChatFilterTitle,
        icon_emoji: impl Into<String>,
        color_index: Option<u8>,
        flags: ChatFilterFlags,
    ) -> Self {
        Self {
            id,
            title,
            icon_emoji: icon_emoji.into(),
            color_index,
            flags,
            always: HashSet::new(),
            pinned: Vec::new(),
            never: HashSet::new(),
        }
    }

    pub fn with_id(mut self, id: FilterId) -> Self {
        self.id = id;
        self
    }

    pub fn with_title(mut self, title: ChatFilterTitle) -> Self {
        self.title = title;
        self
    }

    pub fn with_color_index(mut self, color_index: Option<u8>) -> Self {
        self.color_index = color_index;
        self
    }

    pub fn with_chatlist(mut self, chatlist: bool, has_my_links: bool) -> Self {
        self.flags.set(ChatFilterFlags::CHATLIST, chatlist);
        self.flags.set(ChatFilterFlags::HAS_MY_LINKS, has_my_links);
        self
    }

    pub fn is_chatlist(&self) -> bool {
        self.flags.contains(ChatFilterFlags::CHATLIST)
    }

    pub fn has_my_links(&self) -> bool {
        self.flags.contains(ChatFilterFlags::HAS_MY_LINKS)
    }

    /// Evaluates whether a chat belongs to this folder, matching `ChatFilter::contains`.
    pub fn contains(&self, chat: &FilterChatDescriptor) -> bool {
        if self.never.contains(&chat.peer_id) {
            return false;
        }
        if self.always.contains(&chat.peer_id) {
            return true;
        }

        if self.flags.contains(ChatFilterFlags::NO_MUTED) && chat.is_muted {
            return false;
        }
        if self.flags.contains(ChatFilterFlags::NO_READ) && !chat.is_unread {
            return false;
        }
        if self.flags.contains(ChatFilterFlags::NO_ARCHIVED) && chat.is_archived {
            return false;
        }

        let type_matches = (self.flags.contains(ChatFilterFlags::CONTACTS)
            && chat.is_user
            && !chat.is_bot
            && chat.is_contact)
            || (self.flags.contains(ChatFilterFlags::NON_CONTACTS)
                && chat.is_user
                && !chat.is_bot
                && !chat.is_contact)
            || (self.flags.contains(ChatFilterFlags::GROUPS) && chat.is_group)
            || (self.flags.contains(ChatFilterFlags::CHANNELS) && chat.is_channel)
            || (self.flags.contains(ChatFilterFlags::BOTS) && chat.is_bot);

        type_matches
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatFilterLink {
    pub id: FilterId,
    pub url: String,
    pub title: String,
    pub chats: Vec<PeerId>,
}

/// Ordered collection of chat folders.
#[derive(Debug, Default)]
pub struct ChatFiltersManager {
    order: Vec<FilterId>,
    filters: HashMap<FilterId, ChatFilter>,
}

impl ChatFiltersManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn upsert(&mut self, filter: ChatFilter) {
        let id = filter.id;
        if !self.filters.contains_key(&id) {
            self.order.push(id);
        }
        self.filters.insert(id, filter);
    }

    pub fn remove(&mut self, id: FilterId) -> Option<ChatFilter> {
        self.order.retain(|&x| x != id);
        self.filters.remove(&id)
    }

    pub fn reorder(&mut self, new_order: Vec<FilterId>) {
        self.order = new_order
            .into_iter()
            .filter(|id| self.filters.contains_key(id))
            .collect();
    }

    pub fn get(&self, id: FilterId) -> Option<&ChatFilter> {
        self.filters.get(&id)
    }

    pub fn list(&self) -> Vec<&ChatFilter> {
        self.order
            .iter()
            .filter_map(|id| self.filters.get(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_peers::{ChannelId, ChatId, UserId};

    fn user_peer(id: u64) -> PeerId {
        PeerId::from_user(UserId::new(id))
    }

    #[test]
    fn test_filter_inclusion_exclusion_and_flags() {
        let mut filter = ChatFilter::new(
            2,
            ChatFilterTitle::new("Unread Contacts", false),
            "💬",
            Some(1),
            ChatFilterFlags::CONTACTS | ChatFilterFlags::NO_READ | ChatFilterFlags::NO_ARCHIVED,
        );

        let alice = user_peer(1);
        let bob = user_peer(2);
        let forced = user_peer(3);
        let blocked = user_peer(4);

        filter.always.insert(forced);
        filter.never.insert(blocked);

        assert!(filter.contains(&FilterChatDescriptor {
            peer_id: alice,
            is_contact: true,
            is_user: true,
            is_group: false,
            is_channel: false,
            is_bot: false,
            is_muted: false,
            is_unread: true,
            is_archived: false,
        }));

        // Read contact excluded by NO_READ
        assert!(!filter.contains(&FilterChatDescriptor {
            peer_id: bob,
            is_contact: true,
            is_user: true,
            is_group: false,
            is_channel: false,
            is_bot: false,
            is_muted: false,
            is_unread: false,
            is_archived: false,
        }));

        // Always overrides flags
        assert!(filter.contains(&FilterChatDescriptor {
            peer_id: forced,
            is_contact: false,
            is_user: false,
            is_group: true,
            is_channel: false,
            is_bot: false,
            is_muted: true,
            is_unread: false,
            is_archived: true,
        }));

        // Never overrides everything
        assert!(!filter.contains(&FilterChatDescriptor {
            peer_id: blocked,
            is_contact: true,
            is_user: true,
            is_group: false,
            is_channel: false,
            is_bot: false,
            is_muted: false,
            is_unread: true,
            is_archived: false,
        }));
    }

    #[test]
    fn test_manager_order_and_channels() {
        let mut mgr = ChatFiltersManager::new();
        mgr.upsert(ChatFilter::new(
            2,
            ChatFilterTitle::new("Channels", true),
            "📢",
            None,
            ChatFilterFlags::CHANNELS,
        ));
        mgr.upsert(ChatFilter::new(
            3,
            ChatFilterTitle::new("Groups", false),
            "👥",
            None,
            ChatFilterFlags::GROUPS,
        ));
        mgr.reorder(vec![3, 2]);
        assert_eq!(mgr.list().iter().map(|f| f.id).collect::<Vec<_>>(), vec![3, 2]);

        let ch_filter = mgr.get(2).unwrap();
        assert!(ch_filter.contains(&FilterChatDescriptor {
            peer_id: PeerId::from_channel(ChannelId::new(10)),
            is_contact: false,
            is_user: false,
            is_group: false,
            is_channel: true,
            is_bot: false,
            is_muted: false,
            is_unread: false,
            is_archived: false,
        }));
        assert!(!ch_filter.contains(&FilterChatDescriptor {
            peer_id: PeerId::from_chat(ChatId::new(11)),
            is_contact: false,
            is_user: false,
            is_group: true,
            is_channel: false,
            is_bot: false,
            is_muted: false,
            is_unread: false,
            is_archived: false,
        }));
    }
}
