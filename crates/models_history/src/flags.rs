//! Message flags and MsgId definitions ported from Telegram Desktop.
//!
//! Source: `tdesktop/Telegram/SourceFiles/data/data_types.h`.

pub type MsgId = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MessageFlags(pub u64);

#[allow(non_upper_case_globals)]
impl MessageFlags {
    pub const HideEdited: u64 = 1 << 0;
    pub const Legacy: u64 = 1 << 1;
    pub const HasReplyMarkup: u64 = 1 << 2;
    pub const HasFromId: u64 = 1 << 3;
    pub const HasPostAuthor: u64 = 1 << 4;
    pub const HasViews: u64 = 1 << 5;
    pub const HasReplyInfo: u64 = 1 << 6;
    pub const CanViewReactions: u64 = 1 << 7;
    pub const AdminLogEntry: u64 = 1 << 8;
    pub const Post: u64 = 1 << 9;
    pub const Silent: u64 = 1 << 10;
    pub const Outgoing: u64 = 1 << 11;
    pub const Pinned: u64 = 1 << 12;
    pub const MediaIsUnread: u64 = 1 << 13;
    pub const HasUnreadReaction: u64 = 1 << 14;
    pub const MentionsMe: u64 = 1 << 15;
    pub const IsOrWasScheduled: u64 = 1 << 16;
    pub const NoForwards: u64 = 1 << 17;
    pub const InvertMedia: u64 = 1 << 18;
    pub const HasSwitchInlineButton: u64 = 1 << 19;
    pub const HasTextLinks: u64 = 1 << 20;
    pub const IsGroupEssential: u64 = 1 << 21;
    pub const IsLocalUpdateMedia: u64 = 1 << 22;
    pub const FromInlineBot: u64 = 1 << 23;
    pub const ClientSideUnread: u64 = 1 << 24;
    pub const HasAdminBadge: u64 = 1 << 25;
    pub const BeingSent: u64 = 1 << 26;
    pub const SendingFailed: u64 = 1 << 27;
    pub const EmojiInteractionWatched: u64 = 1 << 28;
    pub const HistoryEntry: u64 = 1 << 29;
    pub const Local: u64 = 1 << 30;
    pub const FakeHistoryItem: u64 = 1 << 31;
    pub const IsContactSignUp: u64 = 1 << 32;
    pub const CustomEmojiRepainting: u64 = 1 << 33;
    pub const IsUserpicSuggestion: u64 = 1 << 34;
    pub const OnlyEmojiAndSpaces: u64 = 1 << 35;
    pub const OnlyEmojiAndSpacesSet: u64 = 1 << 36;
    pub const FakeAboutView: u64 = 1 << 37;
    pub const StoryItem: u64 = 1 << 38;
    pub const InHighlightProcess: u64 = 1 << 39;
    pub const DisplayFromChecked: u64 = 1 << 40;
    pub const DisplayFromProfiles: u64 = 1 << 41;
    pub const ShowSimilarChannels: u64 = 1 << 42;
    pub const Sponsored: u64 = 1 << 43;
    pub const ReactionsAreTags: u64 = 1 << 44;
    pub const ShortcutMessage: u64 = 1 << 45;
    pub const EffectWatched: u64 = 1 << 46;
    pub const SensitiveContent: u64 = 1 << 47;
    pub const HasRestrictions: u64 = 1 << 48;
    pub const EstimatedDate: u64 = 1 << 49;
    pub const ReactionsAllowed: u64 = 1 << 50;
    pub const HideDisplayDate: u64 = 1 << 51;
    pub const StarsPaidSuggested: u64 = 1 << 52;
    pub const TonPaidSuggested: u64 = 1 << 53;
    pub const StoryInProfile: u64 = 1 << 54;
    pub const SavedMusicItem: u64 = 1 << 55;
    pub const HasHiddenLinks: u64 = 1 << 56;
    pub const HasSummaryEntry: u64 = 1 << 57;
    pub const CanBeSummarized: u64 = 1 << 58;
    pub const HasUnreadPollVote: u64 = 1 << 59;
    pub const TextAppearing: u64 = 1 << 60;
    pub const TextAppearingStarted: u64 = 1 << 61;
    pub const GuestChatViaFrom: u64 = 1 << 62;
    pub const Ephemeral: u64 = 1 << 63;

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
