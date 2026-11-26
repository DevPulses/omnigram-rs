//! 1-to-1 voice and video call state machine.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/calls/calls_call.{h,cpp}`.

use models_peers::{PeerId, UserId};

pub type CallId = u64;

pub const SOUND_SAMPLE_MS: u64 = 100;
pub const SIGNAL_BAR_STARTING: i32 = -1;
pub const SIGNAL_BAR_FINISHED: i32 = -2;
pub const SIGNAL_BAR_COUNT: i32 = 4;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DhConfig {
    pub version: i32,
    pub g: i32,
    pub p: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallType {
    Incoming,
    Outgoing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallState {
    Starting,
    WaitingInit,
    WaitingInitAck,
    Established,
    FailedHangingUp,
    Failed,
    HangingUp,
    MigrationHangingUp,
    Ended,
    EndedByOtherDevice,
    ExchangingKeys,
    Waiting,
    Requesting,
    WaitingIncoming,
    Ringing,
    Busy,
    WaitingUserConfirmation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteAudioState {
    Muted,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteVideoState {
    Inactive,
    Paused,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteBatteryState {
    Low,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallErrorType {
    NoCamera,
    NoMicrophone,
    NotStartedCall,
    NotVideoCall,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallError {
    pub error_type: CallErrorType,
    pub details: String,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub id: CallId,
    pub user_id: UserId,
    pub call_type: CallType,
    pub is_video: bool,
    pub state: CallState,
    pub muted: bool,
    pub remote_audio_state: RemoteAudioState,
    pub remote_video_state: RemoteVideoState,
    pub remote_battery_state: RemoteBatteryState,
    pub signal_bars: i32,
    pub duration_ms: u64,
    pub conference_id: CallId,
    pub conference_participants: Vec<PeerId>,
    pub rating: Option<u8>,
}

impl Call {
    pub fn new(id: CallId, user_id: UserId, call_type: CallType, is_video: bool) -> Self {
        let initial_state = match call_type {
            CallType::Incoming => CallState::WaitingIncoming,
            CallType::Outgoing => CallState::Starting,
        };
        Self {
            id,
            user_id,
            call_type,
            is_video,
            state: initial_state,
            muted: false,
            remote_audio_state: RemoteAudioState::Active,
            remote_video_state: if is_video {
                RemoteVideoState::Active
            } else {
                RemoteVideoState::Inactive
            },
            remote_battery_state: RemoteBatteryState::Normal,
            signal_bars: SIGNAL_BAR_STARTING,
            duration_ms: 0,
            conference_id: 0,
            conference_participants: Vec::new(),
            rating: None,
        }
    }

    pub fn is_incoming_waiting(&self) -> bool {
        self.call_type == CallType::Incoming
            && matches!(
                self.state,
                CallState::WaitingIncoming | CallState::WaitingUserConfirmation
            )
    }

    pub fn start_outgoing(&mut self) {
        if self.call_type == CallType::Outgoing && self.state == CallState::Starting {
            self.state = CallState::Requesting;
        }
    }

    pub fn on_ringing(&mut self) {
        if self.call_type == CallType::Outgoing && self.state == CallState::Requesting {
            self.state = CallState::Ringing;
        }
    }

    pub fn answer(&mut self) {
        if self.is_incoming_waiting() {
            self.state = CallState::ExchangingKeys;
        }
    }

    pub fn on_established(&mut self) {
        self.state = CallState::Established;
        self.signal_bars = SIGNAL_BAR_COUNT;
    }

    pub fn hangup(&mut self, busy: bool) {
        if matches!(
            self.state,
            CallState::Ended | CallState::EndedByOtherDevice | CallState::Failed
        ) {
            return;
        }
        self.state = if busy {
            CallState::Busy
        } else {
            CallState::Ended
        };
        self.signal_bars = SIGNAL_BAR_FINISHED;
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn set_rating(&mut self, stars: u8) {
        self.rating = Some(stars.clamp(1, 5));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outgoing_call_lifecycle() {
        let mut call = Call::new(1001, UserId::new(42), CallType::Outgoing, true);
        assert_eq!(call.state, CallState::Starting);
        assert!(!call.is_incoming_waiting());

        call.start_outgoing();
        assert_eq!(call.state, CallState::Requesting);
        call.on_ringing();
        assert_eq!(call.state, CallState::Ringing);
        call.on_established();
        assert_eq!(call.state, CallState::Established);
        assert_eq!(call.signal_bars, SIGNAL_BAR_COUNT);

        call.set_muted(true);
        assert!(call.muted);

        call.hangup(false);
        assert_eq!(call.state, CallState::Ended);
        assert_eq!(call.signal_bars, SIGNAL_BAR_FINISHED);
        call.set_rating(5);
        assert_eq!(call.rating, Some(5));
    }

    #[test]
    fn test_incoming_call_answer() {
        let mut call = Call::new(2002, UserId::new(77), CallType::Incoming, false);
        assert!(call.is_incoming_waiting());
        call.answer();
        assert_eq!(call.state, CallState::ExchangingKeys);
        assert!(!call.is_incoming_waiting());
    }
}
