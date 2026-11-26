//! Reactions registry and per-message reaction state transitions.
//!
//! Source: `tdesktop/Telegram/SourceFiles/data/data_message_reactions.{h,cpp}`.

use std::collections::HashMap;

use data_media_types::DocumentId;
use models_peers::PeerId;

use crate::reaction_id::{MessageReaction, ReactionId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaction {
    pub id: ReactionId,
    pub title: String,
    pub appear_animation: DocumentId,
    pub select_animation: DocumentId,
    pub center_icon: Option<DocumentId>,
    pub around_animation: Option<DocumentId>,
    pub count: i32,
    pub active: bool,
    pub effect: bool,
    pub premium: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MyTagInfo {
    pub id: ReactionId,
    pub title: String,
    pub count: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaidReactionSend {
    pub count: i32,
    pub valid: bool,
    pub shown_peer: Option<PeerId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReactionListType {
    Active,
    Recent,
    Top,
    All,
    MyTags,
    Tags,
    Effects,
}

/// Manages available reactions, favorites, and personal saved message tags.
#[derive(Debug, Default)]
pub struct ReactionsRegistry {
    lists: HashMap<ReactionListType, Vec<Reaction>>,
    favorite: Option<ReactionId>,
    my_tags: HashMap<ReactionId, MyTagInfo>,
}

impl ReactionsRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_list(&mut self, list_type: ReactionListType, items: Vec<Reaction>) {
        self.lists.insert(list_type, items);
    }

    pub fn list(&self, list_type: ReactionListType) -> &[Reaction] {
        self.lists
            .get(&list_type)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn set_favorite(&mut self, id: ReactionId) {
        if !id.is_empty() {
            self.favorite = Some(id);
        }
    }

    pub fn favorite_id(&self) -> Option<&ReactionId> {
        self.favorite.as_ref()
    }

    pub fn increment_my_tag(&mut self, id: ReactionId, title: String) {
        let entry = self.my_tags.entry(id.clone()).or_insert(MyTagInfo {
            id,
            title,
            count: 0,
        });
        entry.count += 1;
    }

    pub fn decrement_my_tag(&mut self, id: &ReactionId) {
        if let Some(entry) = self.my_tags.get_mut(id) {
            entry.count = (entry.count - 1).max(0);
        }
    }

    pub fn rename_tag(&mut self, id: &ReactionId, new_title: String) {
        if let Some(entry) = self.my_tags.get_mut(id) {
            entry.title = new_title;
        }
    }

    pub fn my_tag_title(&self, id: &ReactionId) -> Option<&str> {
        self.my_tags.get(id).map(|t| t.title.as_str())
    }
}

/// Per-message reactions state with toggle & sorting semantics.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MessageReactionsState {
    pub reactions: Vec<MessageReaction>,
    pub recent_reactors: HashMap<ReactionId, Vec<PeerId>>,
    pub can_see_list: bool,
}

impl MessageReactionsState {
    pub fn new(can_see_list: bool) -> Self {
        Self {
            reactions: Vec::new(),
            recent_reactors: HashMap::new(),
            can_see_list,
        }
    }

    /// Toggles the current user's reaction on a message.
    /// Returns `true` if the reaction was added, `false` if removed.
    pub fn toggle_my_reaction(&mut self, id: ReactionId) -> bool {
        if let Some(pos) = self.reactions.iter().position(|r| r.id == id) {
            let entry = &mut self.reactions[pos];
            if entry.my {
                entry.my = false;
                entry.count -= 1;
                if entry.count <= 0 {
                    self.reactions.remove(pos);
                }
                return false;
            } else {
                entry.my = true;
                entry.count += 1;
                self.sort_by_count();
                return true;
            }
        }

        self.reactions.push(MessageReaction::new(id, 1, true));
        self.sort_by_count();
        true
    }

    pub fn total_count(&self) -> i32 {
        self.reactions.iter().map(|r| r.count).sum()
    }

    pub fn chosen(&self) -> Vec<&ReactionId> {
        self.reactions
            .iter()
            .filter(|r| r.my)
            .map(|r| &r.id)
            .collect()
    }

    fn sort_by_count(&mut self) {
        self.reactions.sort_by(|a, b| b.count.cmp(&a.count));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reaction_id_variants() {
        let paid = ReactionId::paid();
        assert!(paid.is_paid());
        assert_eq!(paid.emoji(), None);
        assert_eq!(paid.custom(), None);

        let fire = ReactionId::emoji_id("🔥");
        assert!(!fire.is_paid());
        assert!(!fire.is_empty());
        assert_eq!(fire.emoji(), Some("🔥"));

        let custom = ReactionId::custom_id(987654321);
        assert_eq!(custom.custom(), Some(987654321));
        assert_eq!(custom.emoji(), None);
    }

    #[test]
    fn test_toggle_reactions_and_sorting() {
        let mut state = MessageReactionsState::new(true);
        let thumbs = ReactionId::emoji_id("👍");
        let heart = ReactionId::emoji_id("❤️");

        assert!(state.toggle_my_reaction(thumbs.clone()));
        state.reactions.push(MessageReaction::new(heart.clone(), 5, false));
        state.toggle_my_reaction(heart.clone());

        assert_eq!(state.total_count(), 7);
        assert_eq!(state.reactions[0].id, heart);
        assert_eq!(state.reactions[0].count, 6);
        assert!(state.reactions[0].my);

        // Toggle off thumbs removes it completely since count becomes 0
        assert!(!state.toggle_my_reaction(thumbs.clone()));
        assert_eq!(state.reactions.len(), 1);
    }

    #[test]
    fn test_registry_my_tags() {
        let mut reg = ReactionsRegistry::new();
        let tag = ReactionId::emoji_id("⭐");
        reg.set_favorite(tag.clone());
        assert_eq!(reg.favorite_id(), Some(&tag));

        reg.increment_my_tag(tag.clone(), "Important".to_string());
        reg.rename_tag(&tag, "Pinned".to_string());
        assert_eq!(reg.my_tag_title(&tag), Some("Pinned"));
    }
}
