use models_peers::PeerId;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SendMessageRequest {
    pub peer_id: PeerId,
    pub message: String,
    pub random_id: u64,
    pub reply_to_msg_id: Option<i32>,
    pub clear_draft: bool,
    pub silent: bool,
    pub schedule_date: Option<i32>,
}

impl SendMessageRequest {
    pub fn new(peer_id: PeerId, message: String) -> Self {
        let mut rng = rand::thread_rng();
        let random_id: u64 = rng.gen();
        Self {
            peer_id,
            message,
            random_id,
            reply_to_msg_id: None,
            clear_draft: true,
            silent: false,
            schedule_date: None,
        }
    }

    pub fn with_reply(mut self, reply_id: i32) -> Self {
        self.reply_to_msg_id = Some(reply_id);
        self
    }

    pub fn with_silent(mut self, silent: bool) -> Self {
        self.silent = silent;
        self
    }
}
