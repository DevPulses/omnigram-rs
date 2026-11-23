//! `models_peers` — Peer models and identifiers ported from Telegram Desktop.
//!
//! Source of truth:
//! - `tdesktop/Telegram/SourceFiles/data/data_peer_id.{h,cpp}`
//! - `tdesktop/Telegram/SourceFiles/data/data_peer.{h,cpp}`
//! - `tdesktop/Telegram/SourceFiles/data/data_user.{h,cpp}`
//! - `tdesktop/Telegram/SourceFiles/data/data_chat.{h,cpp}`
//! - `tdesktop/Telegram/SourceFiles/data/data_channel.{h,cpp}`
//! - `tdesktop/Telegram/SourceFiles/data/data_peer_common.h`
//! - `tdesktop/Telegram/SourceFiles/data/data_peer_colors.h`

pub mod id;
pub mod flags;
pub mod data;

pub use id::{
    ChannelId, ChatId, FakeChatId, PeerId, UserId,
    deserialize_peer_id, serialize_peer_id,
};
pub use flags::{
    ChannelDataFlag, ChannelDataFlags, ChatDataFlag, ChatDataFlags,
    UserDataFlag, UserDataFlags,
};
pub use data::{
    ChannelData, ChatData, PeerData, PeerDataEnum, UserData,
    BotInfo,
};
