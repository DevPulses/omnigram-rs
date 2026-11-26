//! Chat folders (dialog filters) model and matching rules.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/data/data_chat_filters.{h,cpp}`.

pub mod filter;

pub use filter::{
    ChatFilter, ChatFilterFlags, ChatFilterLink, ChatFilterTitle, ChatFiltersManager,
    FilterChatDescriptor, FilterId, MAIN_FILTER_ID,
};
