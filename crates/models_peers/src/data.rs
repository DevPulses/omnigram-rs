//! Data structures for UserData, ChatData, ChannelData, PeerData.
//!
//! Source: `data_peer.h`, `data_user.h`, `data_chat.h`, `data_channel.h`.

use crate::flags::{ChannelDataFlags, ChatDataFlags, UserDataFlags};
use crate::id::{ChannelId, ChatId, PeerId, UserId};

/// Bot info structure.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BotInfo {
    pub version: i32,
    pub inline_placeholder: String,
    pub supports_guest_chat: bool,
    pub description: String,
}

/// User details in `UserData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserData {
    pub id: UserId,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub username: String,
    pub access_hash: u64,
    pub flags: UserDataFlags,
    pub lastseen: u32,
    pub is_contact: bool,
    pub bot_info: Option<BotInfo>,
}

impl UserData {
    pub fn new(id: UserId) -> Self {
        Self {
            id,
            first_name: String::new(),
            last_name: String::new(),
            phone: String::new(),
            username: String::new(),
            access_hash: 0,
            flags: UserDataFlags::empty(),
            lastseen: 0,
            is_contact: false,
            bot_info: None,
        }
    }

    pub fn peer_id(&self) -> PeerId {
        PeerId::from_user(self.id)
    }

    pub fn is_bot(&self) -> bool {
        self.bot_info.is_some()
    }

    pub fn full_name(&self) -> String {
        if self.first_name.is_empty() {
            self.last_name.clone()
        } else if self.last_name.is_empty() {
            self.first_name.clone()
        } else {
            format!("{} {}", self.first_name, self.last_name)
        }
    }
}

/// Chat (small group) details in `ChatData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatData {
    pub id: ChatId,
    pub name: String,
    pub count: i32,
    pub date: i32,
    pub version: i32,
    pub creator: UserId,
    pub flags: ChatDataFlags,
    pub invite_link: String,
}

impl ChatData {
    pub fn new(id: ChatId) -> Self {
        Self {
            id,
            name: String::new(),
            count: 0,
            date: 0,
            version: 0,
            creator: UserId::new(0),
            flags: ChatDataFlags::empty(),
            invite_link: String::new(),
        }
    }

    pub fn peer_id(&self) -> PeerId {
        PeerId::from_chat(self.id)
    }

    pub fn is_forbidden(&self) -> bool {
        self.flags.contains(ChatDataFlags::Forbidden)
    }

    pub fn am_creator(&self) -> bool {
        self.flags.contains(ChatDataFlags::Creator)
    }
}

/// Channel / Megagroup details in `ChannelData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelData {
    pub id: ChannelId,
    pub name: String,
    pub access_hash: u64,
    pub date: i32,
    pub flags: ChannelDataFlags,
    pub invite_link: String,
}

impl ChannelData {
    pub fn new(id: ChannelId) -> Self {
        Self {
            id,
            name: String::new(),
            access_hash: 0,
            date: 0,
            flags: ChannelDataFlags::empty(),
            invite_link: String::new(),
        }
    }

    pub fn peer_id(&self) -> PeerId {
        PeerId::from_channel(self.id)
    }

    pub fn is_megagroup(&self) -> bool {
        self.flags.contains(ChannelDataFlags::Megagroup)
    }

    pub fn is_broadcast(&self) -> bool {
        self.flags.contains(ChannelDataFlags::Broadcast)
    }

    pub fn is_forbidden(&self) -> bool {
        self.flags.contains(ChannelDataFlags::Forbidden)
    }
}

/// Enum distinguishing specific peer types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PeerDataEnum {
    User(UserData),
    Chat(ChatData),
    Channel(ChannelData),
    Empty(PeerId),
}

/// Generic `PeerData` containing common information across all peers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PeerData {
    pub id: PeerId,
    pub userpic_photo_id: u64,
    pub userpic_has_video: bool,
    pub color_index: u8,
    pub data: PeerDataEnum,
}

impl PeerData {
    pub fn new(id: PeerId) -> Self {
        let data = if id.is_user() {
            PeerDataEnum::User(UserData::new(id.to_user().unwrap_or_default()))
        } else if id.is_chat() {
            PeerDataEnum::Chat(ChatData::new(id.to_chat().unwrap_or_default()))
        } else if id.is_channel() {
            PeerDataEnum::Channel(ChannelData::new(id.to_channel().unwrap_or_default()))
        } else {
            PeerDataEnum::Empty(id)
        };

        Self {
            id,
            userpic_photo_id: 0,
            userpic_has_video: false,
            color_index: decide_color_index(id),
            data,
        }
    }

    pub fn as_user(&self) -> Option<&UserData> {
        match &self.data {
            PeerDataEnum::User(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_user_mut(&mut self) -> Option<&mut UserData> {
        match &mut self.data {
            PeerDataEnum::User(u) => Some(u),
            _ => None,
        }
    }

    pub fn as_chat(&self) -> Option<&ChatData> {
        match &self.data {
            PeerDataEnum::Chat(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_chat_mut(&mut self) -> Option<&mut ChatData> {
        match &mut self.data {
            PeerDataEnum::Chat(c) => Some(c),
            _ => None,
        }
    }

    pub fn as_channel(&self) -> Option<&ChannelData> {
        match &self.data {
            PeerDataEnum::Channel(ch) => Some(ch),
            _ => None,
        }
    }

    pub fn as_channel_mut(&mut self) -> Option<&mut ChannelData> {
        match &mut self.data {
            PeerDataEnum::Channel(ch) => Some(ch),
            _ => None,
        }
    }

    pub fn name(&self) -> &str {
        match &self.data {
            PeerDataEnum::User(u) => {
                if !u.first_name.is_empty() {
                    &u.first_name
                } else if !u.last_name.is_empty() {
                    &u.last_name
                } else {
                    &u.phone
                }
            }
            PeerDataEnum::Chat(c) => &c.name,
            PeerDataEnum::Channel(ch) => &ch.name,
            PeerDataEnum::Empty(_) => "",
        }
    }
}

/// Computes the default color index for a peer.
///
/// Port of `DecideColorIndex` from `data_peer.cpp`.
pub fn decide_color_index(peer_id: PeerId) -> u8 {
    let bare = peer_id.bare_id();
    (bare % 7) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_data_creation() {
        let u_peer = PeerData::new(PeerId::from_user(UserId::new(12345)));
        assert!(u_peer.as_user().is_some());
        assert!(u_peer.as_chat().is_none());
        assert_eq!(u_peer.color_index, (12345 % 7) as u8);

        let c_peer = PeerData::new(PeerId::from_chat(ChatId::new(456)));
        assert!(c_peer.as_chat().is_some());

        let ch_peer = PeerData::new(PeerId::from_channel(ChannelId::new(789)));
        assert!(ch_peer.as_channel().is_some());
    }
}
