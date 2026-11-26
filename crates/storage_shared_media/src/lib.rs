//! Shared media index per peer/topic and sparse IDs querying.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/storage/storage_shared_media.{h,cpp}`.

use std::collections::{BTreeSet, HashMap};

use models_history::flags::MsgId;
use models_peers::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum SharedMediaType {
    Photo = 0,
    Video = 1,
    PhotoVideo = 2,
    MusicFile = 3,
    File = 4,
    VoiceFile = 5,
    Link = 6,
    ChatPhoto = 7,
    RoundVoiceFile = 8,
    GIF = 9,
    RoundFile = 10,
    Pinned = 11,
    Poll = 12,
}

pub const SHARED_MEDIA_TYPE_COUNT: usize = 13;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct SharedMediaTypesMask: u32 {
        const PHOTO            = 1 << 0;
        const VIDEO            = 1 << 1;
        const PHOTO_VIDEO      = 1 << 2;
        const MUSIC_FILE       = 1 << 3;
        const FILE             = 1 << 4;
        const VOICE_FILE       = 1 << 5;
        const LINK             = 1 << 6;
        const CHAT_PHOTO       = 1 << 7;
        const ROUND_VOICE_FILE = 1 << 8;
        const GIF              = 1 << 9;
        const ROUND_FILE       = 1 << 10;
        const PINNED           = 1 << 11;
        const POLL             = 1 << 12;
    }
}

impl SharedMediaTypesMask {
    pub fn from_type(media_type: SharedMediaType) -> Self {
        Self::from_bits_truncate(1 << (media_type as u32))
    }

    pub fn contains_type(self, media_type: SharedMediaType) -> bool {
        self.contains(Self::from_type(media_type))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SharedMediaKey {
    pub peer_id: PeerId,
    pub topic_root_id: MsgId,
    pub media_type: SharedMediaType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedMediaSlice {
    pub ids: Vec<MsgId>,
    pub skipped_before: usize,
    pub skipped_after: usize,
    pub full_count: usize,
}

/// In-memory sparse index of shared media message IDs per `(peer, topic, media_type)`.
#[derive(Debug, Default)]
pub struct SharedMediaStorage {
    lists: HashMap<SharedMediaKey, BTreeSet<MsgId>>,
}

impl SharedMediaStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_new(
        &mut self,
        peer_id: PeerId,
        topic_root_id: MsgId,
        msg_id: MsgId,
        types: SharedMediaTypesMask,
    ) {
        for bit in 0..SHARED_MEDIA_TYPE_COUNT {
            let flag = SharedMediaTypesMask::from_bits_truncate(1 << bit);
            if types.contains(flag) {
                let media_type = match bit {
                    0 => SharedMediaType::Photo,
                    1 => SharedMediaType::Video,
                    2 => SharedMediaType::PhotoVideo,
                    3 => SharedMediaType::MusicFile,
                    4 => SharedMediaType::File,
                    5 => SharedMediaType::VoiceFile,
                    6 => SharedMediaType::Link,
                    7 => SharedMediaType::ChatPhoto,
                    8 => SharedMediaType::RoundVoiceFile,
                    9 => SharedMediaType::GIF,
                    10 => SharedMediaType::RoundFile,
                    11 => SharedMediaType::Pinned,
                    12 => SharedMediaType::Poll,
                    _ => unreachable!(),
                };
                self.lists
                    .entry(SharedMediaKey {
                        peer_id,
                        topic_root_id,
                        media_type,
                    })
                    .or_default()
                    .insert(msg_id);
            }
        }
    }

    pub fn remove_one(&mut self, peer_id: PeerId, topic_root_id: MsgId, msg_id: MsgId) {
        for (key, set) in &mut self.lists {
            if key.peer_id == peer_id && key.topic_root_id == topic_root_id {
                set.remove(&msg_id);
            }
        }
    }

    pub fn count(&self, key: SharedMediaKey) -> usize {
        self.lists.get(&key).map(|s| s.len()).unwrap_or(0)
    }

    pub fn query_around(
        &self,
        key: SharedMediaKey,
        around_id: MsgId,
        limit_before: usize,
        limit_after: usize,
    ) -> SharedMediaSlice {
        let Some(set) = self.lists.get(&key) else {
            return SharedMediaSlice {
                ids: Vec::new(),
                skipped_before: 0,
                skipped_after: 0,
                full_count: 0,
            };
        };

        let all: Vec<MsgId> = set.iter().copied().collect();
        let full_count = all.len();
        let pivot = all.partition_point(|&id| id < around_id);
        let start = pivot.saturating_sub(limit_before);
        let end = (pivot + limit_after + 1).min(full_count);

        SharedMediaSlice {
            ids: all[start..end].to_vec(),
            skipped_before: start,
            skipped_after: full_count.saturating_sub(end),
            full_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_peers::UserId;

    #[test]
    fn test_shared_media_add_query_and_remove() {
        let mut storage = SharedMediaStorage::new();
        let peer = PeerId::from_user(UserId::new(10));
        let mask = SharedMediaTypesMask::PHOTO | SharedMediaTypesMask::PHOTO_VIDEO;

        for id in [10, 20, 30, 40, 50] {
            storage.add_new(peer, 0, id, mask);
        }

        let key = SharedMediaKey {
            peer_id: peer,
            topic_root_id: 0,
            media_type: SharedMediaType::Photo,
        };
        assert_eq!(storage.count(key), 5);

        let slice = storage.query_around(key, 30, 1, 1);
        assert_eq!(slice.ids, vec![20, 30, 40]);
        assert_eq!(slice.skipped_before, 1);
        assert_eq!(slice.skipped_after, 1);
        assert_eq!(slice.full_count, 5);

        storage.remove_one(peer, 0, 30);
        assert_eq!(storage.count(key), 4);
    }
}
