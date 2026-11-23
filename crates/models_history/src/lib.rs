//! `models_history` — Message and history models ported from Telegram Desktop.
//!
//! Source:
//! - `tdesktop/Telegram/SourceFiles/history/history_item.{h,cpp}`
//! - `tdesktop/Telegram/SourceFiles/data/data_types.h`

pub mod flags;
pub mod item;

pub use flags::{MessageFlags, MsgId};
pub use item::{History, HistoryItem, InlineButton, InlineButtonRow, ReplyToInfo};
