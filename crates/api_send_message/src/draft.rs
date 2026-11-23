use models_peers::PeerId;


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DraftKey {
    pub peer_id: PeerId,
    pub topic_id: i32,
}

impl DraftKey {
    pub fn new(peer_id: PeerId, topic_id: i32) -> Self {
        Self { peer_id, topic_id }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft {
    pub text: String,
    pub reply_to_msg_id: Option<i32>,
    pub cursor_position: i32,
    pub date: i32,
}

impl Draft {
    pub fn new(text: String, reply_to_msg_id: Option<i32>, cursor_position: i32, date: i32) -> Self {
        Self {
            text,
            reply_to_msg_id,
            cursor_position,
            date,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && self.reply_to_msg_id.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::progress::*;
    use crate::send_action::*;

    #[test]
    fn test_send_progress_rate_limit() {
        let mut manager = SendProgressManager::new();
        let peer = PeerId::from_user(models_peers::UserId::new(42));

        // First update goes through
        assert!(manager.update(peer, 0, SendProgressType::Typing, 1000, 0));
        assert!(manager.has_active(peer));

        // Sub-4000ms update is rate limited
        assert!(!manager.update(peer, 0, SendProgressType::Typing, 2000, 0));

        // After 4000ms update succeeds
        assert!(manager.update(peer, 0, SendProgressType::Typing, 5001, 0));

        manager.cancel(peer, 0, SendProgressType::Typing);
        assert!(!manager.has_active(peer));
    }

    #[test]
    fn test_send_message_request() {
        let peer = PeerId::from_chat(models_peers::ChatId::new(100));
        let req = SendMessageRequest::new(peer, "Hello Rust!".to_string())
            .with_reply(10)
            .with_silent(true);

        assert_eq!(req.peer_id, peer);
        assert_eq!(req.message, "Hello Rust!");
        assert_eq!(req.reply_to_msg_id, Some(10));
        assert!(req.silent);
        assert_ne!(req.random_id, 0);
    }

    #[test]
    fn test_draft_lifecycle() {
        let draft = Draft::new("Typing something...".to_string(), None, 5, 1700000000);
        assert!(!draft.is_empty());

        let empty_draft = Draft::new("   ".to_string(), None, 0, 1700000000);
        assert!(empty_draft.is_empty());
    }
}
