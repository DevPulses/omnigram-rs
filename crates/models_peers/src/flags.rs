//! Flags ported from tdesktop peer models.
//!
//! Source: `data_user.h`, `data_chat.h`, `data_channel.h`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct UserDataFlags(pub u32);

#[allow(non_upper_case_globals)]
impl UserDataFlags {
    pub const Deleted: u32 = 1 << 0;
    pub const Verified: u32 = 1 << 1;
    pub const Scam: u32 = 1 << 2;
    pub const Fake: u32 = 1 << 3;
    pub const BotInlineGeo: u32 = 1 << 4;
    pub const Support: u32 = 1 << 5;
    pub const Contact: u32 = 1 << 6;
    pub const MutualContact: u32 = 1 << 7;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(&self, flag: u32) -> bool {
        (self.0 & flag) == flag
    }

    pub fn insert(&mut self, flag: u32) {
        self.0 |= flag;
    }

    pub fn remove(&mut self, flag: u32) {
        self.0 &= !flag;
    }
}

pub type UserDataFlag = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ChatDataFlags(pub u32);

#[allow(non_upper_case_globals)]
impl ChatDataFlags {
    pub const Left: u32 = 1 << 0;
    pub const Creator: u32 = 1 << 2;
    pub const Deactivated: u32 = 1 << 3;
    pub const Forbidden: u32 = 1 << 4;
    pub const CallActive: u32 = 1 << 5;
    pub const CallNotEmpty: u32 = 1 << 6;
    pub const CanSetUsername: u32 = 1 << 7;
    pub const NoForwards: u32 = 1 << 8;
    pub const HasWelcomeMessages: u32 = 1 << 9;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(&self, flag: u32) -> bool {
        (self.0 & flag) == flag
    }

    pub fn insert(&mut self, flag: u32) {
        self.0 |= flag;
    }

    pub fn remove(&mut self, flag: u32) {
        self.0 &= !flag;
    }
}

pub type ChatDataFlag = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ChannelDataFlags(pub u64);

#[allow(non_upper_case_globals)]
impl ChannelDataFlags {
    pub const Left: u64 = 1 << 0;
    pub const Creator: u64 = 1 << 1;
    pub const Forbidden: u64 = 1 << 2;
    pub const CallActive: u64 = 1 << 3;
    pub const CallNotEmpty: u64 = 1 << 4;
    pub const Signatures: u64 = 1 << 5;
    pub const Verified: u64 = 1 << 6;
    pub const Scam: u64 = 1 << 7;
    pub const Fake: u64 = 1 << 8;
    pub const Megagroup: u64 = 1 << 9;
    pub const Broadcast: u64 = 1 << 10;
    pub const Gigagroup: u64 = 1 << 11;
    pub const Username: u64 = 1 << 12;
    pub const Location: u64 = 1 << 13;
    pub const CanSetUsername: u64 = 1 << 14;
    pub const CanSetStickers: u64 = 1 << 15;
    pub const PreHistoryHidden: u64 = 1 << 16;
    pub const CanViewParticipants: u64 = 1 << 17;
    pub const HasLink: u64 = 1 << 18;
    pub const SlowmodeEnabled: u64 = 1 << 19;
    pub const NoForwards: u64 = 1 << 20;
    pub const JoinToWrite: u64 = 1 << 21;
    pub const RequestToJoin: u64 = 1 << 22;
    pub const Forum: u64 = 1 << 23;
    pub const AntiSpam: u64 = 1 << 24;
    pub const ParticipantsHidden: u64 = 1 << 25;
    pub const StoriesHidden: u64 = 1 << 26;
    pub const HasActiveStories: u64 = 1 << 27;
    pub const HasUnreadStories: u64 = 1 << 28;
    pub const CanGetStatistics: u64 = 1 << 29;
    pub const ViewAsMessages: u64 = 1 << 30;
    pub const SimilarExpanded: u64 = 1 << 31;
    pub const CanViewRevenue: u64 = 1 << 32;
    pub const PaidMediaAllowed: u64 = 1 << 33;
    pub const CanViewCreditsRevenue: u64 = 1 << 34;
    pub const SignatureProfiles: u64 = 1 << 35;
    pub const StargiftsAvailable: u64 = 1 << 36;
    pub const PaidMessagesAvailable: u64 = 1 << 37;
    pub const AutoTranslation: u64 = 1 << 38;
    pub const Monoforum: u64 = 1 << 39;
    pub const MonoforumAdmin: u64 = 1 << 40;
    pub const MonoforumDisabled: u64 = 1 << 41;
    pub const ForumTabs: u64 = 1 << 42;
    pub const HasStarsPerMessage: u64 = 1 << 43;
    pub const StarsPerMessageKnown: u64 = 1 << 44;
    pub const HasActiveVideoStream: u64 = 1 << 45;
    pub const Community: u64 = 1 << 46;
    pub const CommunityCollapsed: u64 = 1 << 47;
    pub const HasWelcomeMessages: u64 = 1 << 48;

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(&self, flag: u64) -> bool {
        (self.0 & flag) == flag
    }

    pub fn insert(&mut self, flag: u64) {
        self.0 |= flag;
    }

    pub fn remove(&mut self, flag: u64) {
        self.0 &= !flag;
    }
}

pub type ChannelDataFlag = u64;
