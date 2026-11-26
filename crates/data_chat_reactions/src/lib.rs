//! Message reactions model and manager.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/data/data_message_reaction_id.h`
//! and `tdesktop/Telegram/SourceFiles/data/data_message_reactions.{h,cpp}`.

pub mod reaction_id;
pub mod reactions;

pub use reaction_id::{MessageReaction, ReactionId, PAID_TAG};
pub use reactions::{
    MessageReactionsState, MyTagInfo, PaidReactionSend, Reaction, ReactionListType,
    ReactionsRegistry,
};
