use std::collections::HashMap;
use models_peers::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SendProgressType {
    Typing,
    RecordVideo,
    UploadVideo,
    RecordVoice,
    UploadVoice,
    RecordRound,
    UploadRound,
    UploadPhoto,
    UploadFile,
    ChooseLocation,
    ChooseContact,
    ChooseSticker,
    PlayGame,
    Speaking,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendProgress {
    pub progress_type: SendProgressType,
    pub until_ms: u64,
    pub progress: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProgressKey {
    pub peer_id: PeerId,
    pub top_msg_id: i32,
    pub progress_type: SendProgressType,
}

#[derive(Default)]
pub struct SendProgressManager {
    requests: HashMap<ProgressKey, u64>,
    last_updated: HashMap<ProgressKey, u64>,
}

impl SendProgressManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self, peer_id: PeerId, top_msg_id: i32, progress_type: SendProgressType, now_ms: u64, progress: i32) -> bool {
        let key = ProgressKey {
            peer_id,
            top_msg_id,
            progress_type,
        };

        if let Some(&last) = self.last_updated.get(&key) {
            // Rate limit typing updates: don't resend within 4000ms
            if now_ms < last + 4000 && progress == 0 {
                return false;
            }
        }

        self.last_updated.insert(key.clone(), now_ms);
        self.requests.insert(key, now_ms);
        true
    }

    pub fn cancel(&mut self, peer_id: PeerId, top_msg_id: i32, progress_type: SendProgressType) {
        let key = ProgressKey {
            peer_id,
            top_msg_id,
            progress_type,
        };
        self.requests.remove(&key);
        self.last_updated.remove(&key);
    }

    pub fn has_active(&self, peer_id: PeerId) -> bool {
        self.requests.keys().any(|k| k.peer_id == peer_id)
    }
}
