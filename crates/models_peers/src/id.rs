//! PeerId and strongly typed ChatIdType definitions.
//!
//! Port of `tdesktop/Telegram/SourceFiles/data/data_peer_id.{h,cpp}`.

pub type BareId = u64;

/// Shift constants for `ChatIdType`.
pub const USER_ID_SHIFT: u8 = 0;
pub const CHAT_ID_SHIFT: u8 = 1;
pub const CHANNEL_ID_SHIFT: u8 = 2;
pub const FAKE_CHAT_ID_SHIFT: u8 = 0x7F;

/// Reserved bit flag in shift byte: `0x80`.
pub const RESERVED_BIT: BareId = 0x80;

/// Mask for the 48-bit bare identifier part.
pub const CHAT_TYPE_MASK: BareId = 0xFFFF_FFFF_FFFF;

/// Strongly typed identifier wrapper for User, Chat, Channel, FakeChat.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ChatIdType<const SHIFT: u8> {
    pub bare: BareId,
}

impl<const SHIFT: u8> ChatIdType<SHIFT> {
    pub const SHIFT: u8 = SHIFT;

    pub const fn new(bare: BareId) -> Self {
        Self { bare }
    }

    pub const fn is_zero(&self) -> bool {
        self.bare == 0
    }
}

impl<const SHIFT: u8> From<BareId> for ChatIdType<SHIFT> {
    fn from(bare: BareId) -> Self {
        Self { bare }
    }
}

impl<const SHIFT: u8> From<ChatIdType<SHIFT>> for BareId {
    fn from(id: ChatIdType<SHIFT>) -> Self {
        id.bare
    }
}

pub type UserId = ChatIdType<USER_ID_SHIFT>;
pub type ChatId = ChatIdType<CHAT_ID_SHIFT>;
pub type ChannelId = ChatIdType<CHANNEL_ID_SHIFT>;
pub type FakeChatId = ChatIdType<FAKE_CHAT_ID_SHIFT>;

/// `PeerId` representing a Telegram peer (User, Chat, Channel, etc.).
/// Internal 64-bit value: upper 16 bits contain Shift (at bit 48), lower 48 bits contain bare ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PeerId {
    pub value: BareId,
}

impl PeerId {
    pub const ZERO: Self = Self { value: 0 };
    pub const SERVICE_NOTIFICATIONS: Self = Self::from_user(UserId::new(777000));
    pub const SAVED_HIDDEN_AUTHOR: Self = Self::from_user(UserId::new(2666000));

    pub const fn from_raw(value: BareId) -> Self {
        Self { value }
    }

    pub const fn from_user(id: UserId) -> Self {
        Self {
            value: (id.bare & CHAT_TYPE_MASK) | ((USER_ID_SHIFT as BareId) << 48),
        }
    }

    pub const fn from_chat(id: ChatId) -> Self {
        Self {
            value: (id.bare & CHAT_TYPE_MASK) | ((CHAT_ID_SHIFT as BareId) << 48),
        }
    }

    pub const fn from_channel(id: ChannelId) -> Self {
        Self {
            value: (id.bare & CHAT_TYPE_MASK) | ((CHANNEL_ID_SHIFT as BareId) << 48),
        }
    }

    pub const fn from_fake_chat(id: FakeChatId) -> Self {
        Self {
            value: (id.bare & CHAT_TYPE_MASK) | ((FAKE_CHAT_ID_SHIFT as BareId) << 48),
        }
    }

    pub const fn shift(&self) -> u8 {
        ((self.value >> 48) & 0xFF) as u8
    }

    pub const fn bare_id(&self) -> BareId {
        self.value & CHAT_TYPE_MASK
    }

    pub const fn is_user(&self) -> bool {
        self.shift() == USER_ID_SHIFT
    }

    pub const fn is_chat(&self) -> bool {
        self.shift() == CHAT_ID_SHIFT
    }

    pub const fn is_channel(&self) -> bool {
        self.shift() == CHANNEL_ID_SHIFT
    }

    pub const fn is_fake_chat(&self) -> bool {
        self.shift() == FAKE_CHAT_ID_SHIFT
    }

    pub const fn to_user(&self) -> Option<UserId> {
        if self.is_user() {
            Some(UserId::new(self.bare_id()))
        } else {
            None
        }
    }

    pub const fn to_chat(&self) -> Option<ChatId> {
        if self.is_chat() {
            Some(ChatId::new(self.bare_id()))
        } else {
            None
        }
    }

    pub const fn to_channel(&self) -> Option<ChannelId> {
        if self.is_channel() {
            Some(ChannelId::new(self.bare_id()))
        } else {
            None
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.value == 0
    }
}

impl From<UserId> for PeerId {
    fn from(id: UserId) -> Self {
        Self::from_user(id)
    }
}

impl From<ChatId> for PeerId {
    fn from(id: ChatId) -> Self {
        Self::from_chat(id)
    }
}

impl From<ChannelId> for PeerId {
    fn from(id: ChannelId) -> Self {
        Self::from_channel(id)
    }
}

impl From<FakeChatId> for PeerId {
    fn from(id: FakeChatId) -> Self {
        Self::from_fake_chat(id)
    }
}

/// Serializes a `PeerId` into `quint64` with `UserId::kReservedBit` (0x80) flag at bit 48.
pub fn serialize_peer_id(id: PeerId) -> u64 {
    assert_eq!(
        id.value & (RESERVED_BIT << 48),
        0,
        "PeerId already has reserved bit set"
    );
    id.value | (RESERVED_BIT << 48)
}

/// Deserializes a `PeerId` from modern or legacy serialization.
///
/// Port of `tdesktop/Telegram/SourceFiles/data/data_peer_id.cpp:DeserializePeerId`.
pub fn deserialize_peer_id(serialized: u64) -> PeerId {
    let flag = RESERVED_BIT << 48;
    let legacy = (serialized & flag) == 0;
    if !legacy {
        return PeerId::from_raw(serialized & (!flag));
    }

    const PEER_ID_MASK: u64 = 0xFFFF_FFFF;
    const PEER_ID_TYPE_MASK: u64 = 0xF_0000_0000;
    const PEER_ID_USER_SHIFT: u64 = 0x0_0000_0000;
    const PEER_ID_CHAT_SHIFT: u64 = 0x1_0000_0000;
    const PEER_ID_CHANNEL_SHIFT: u64 = 0x2_0000_0000;
    const PEER_ID_FAKE_SHIFT: u64 = 0xF_0000_0000;

    let t = serialized & PEER_ID_TYPE_MASK;
    let bare = serialized & PEER_ID_MASK;

    match t {
        PEER_ID_USER_SHIFT => PeerId::from_user(UserId::new(bare)),
        PEER_ID_CHAT_SHIFT => PeerId::from_chat(ChatId::new(bare)),
        PEER_ID_CHANNEL_SHIFT => PeerId::from_channel(ChannelId::new(bare)),
        PEER_ID_FAKE_SHIFT => PeerId::from_fake_chat(FakeChatId::new(bare)),
        _ => PeerId::ZERO,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id_types() {
        let u = UserId::new(12345);
        let p_u = PeerId::from_user(u);
        assert!(p_u.is_user());
        assert!(!p_u.is_chat());
        assert_eq!(p_u.to_user(), Some(u));
        assert_eq!(p_u.bare_id(), 12345);

        let c = ChatId::new(67890);
        let p_c = PeerId::from_chat(c);
        assert!(p_c.is_chat());
        assert_eq!(p_c.to_chat(), Some(c));
        assert_eq!(p_c.bare_id(), 67890);

        let ch = ChannelId::new(111222);
        let p_ch = PeerId::from_channel(ch);
        assert!(p_ch.is_channel());
        assert_eq!(p_ch.to_channel(), Some(ch));
        assert_eq!(p_ch.bare_id(), 111222);
    }

    #[test]
    fn test_serialize_deserialize_modern() {
        let p_u = PeerId::from_user(UserId::new(42));
        let ser = serialize_peer_id(p_u);
        assert_eq!(deserialize_peer_id(ser), p_u);

        let p_c = PeerId::from_chat(ChatId::new(999));
        let ser_c = serialize_peer_id(p_c);
        assert_eq!(deserialize_peer_id(ser_c), p_c);

        let p_ch = PeerId::from_channel(ChannelId::new(100500));
        let ser_ch = serialize_peer_id(p_ch);
        assert_eq!(deserialize_peer_id(ser_ch), p_ch);
    }

    #[test]
    fn test_deserialize_legacy() {
        // Legacy user 42
        assert_eq!(
            deserialize_peer_id(42),
            PeerId::from_user(UserId::new(42))
        );
        // Legacy chat 42: shift 0x1_0000_0000
        assert_eq!(
            deserialize_peer_id(0x1_0000_0000 | 42),
            PeerId::from_chat(ChatId::new(42))
        );
        // Legacy channel 42: shift 0x2_0000_0000
        assert_eq!(
            deserialize_peer_id(0x2_0000_0000 | 42),
            PeerId::from_channel(ChannelId::new(42))
        );
        // Legacy fake 42: shift 0xF_0000_0000
        assert_eq!(
            deserialize_peer_id(0xF_0000_0000 | 42),
            PeerId::from_fake_chat(FakeChatId::new(42))
        );
    }
}
