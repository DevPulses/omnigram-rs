//! Reaction identifier and per-message reaction count entry.
//!
//! Source: `tdesktop/Telegram/SourceFiles/data/data_message_reaction_id.h`.

use data_media_types::DocumentId;

pub const PAID_TAG: char = '*';

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReactionId {
    Emoji(String),
    Custom(DocumentId),
}

impl Default for ReactionId {
    fn default() -> Self {
        Self::Emoji(String::new())
    }
}

impl ReactionId {
    pub fn emoji_id(emoji: impl Into<String>) -> Self {
        Self::Emoji(emoji.into())
    }

    pub fn custom_id(document_id: DocumentId) -> Self {
        Self::Custom(document_id)
    }

    pub fn paid() -> Self {
        Self::Emoji(PAID_TAG.to_string())
    }

    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Emoji(s) if s.is_empty())
    }

    pub fn is_paid(&self) -> bool {
        matches!(self, Self::Emoji(s) if s == "*")
    }

    pub fn emoji(&self) -> Option<&str> {
        match self {
            Self::Emoji(s) if !s.is_empty() && s != "*" => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn custom(&self) -> Option<DocumentId> {
        match self {
            Self::Custom(id) => Some(*id),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageReaction {
    pub id: ReactionId,
    pub count: i32,
    pub my: bool,
}

impl MessageReaction {
    pub fn new(id: ReactionId, count: i32, my: bool) -> Self {
        Self { id, count, my }
    }
}
