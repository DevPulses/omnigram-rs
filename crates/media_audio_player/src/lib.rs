//! Audio playback state machine, voice/song queue, and waveform metrics.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/media/audio/media_audio.{h,cpp}`.

use data_media_types::DocumentId;

pub const DEFAULT_FREQUENCY: u32 = 48_000;
pub const TOGETHER_LIMIT: usize = 4;
pub const WAVEFORM_SAMPLES_COUNT: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioType {
    Voice,
    Song,
    Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioMsgId {
    pub audio_type: AudioType,
    pub document_id: DocumentId,
    pub msg_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    Stopped = 0x01,
    StoppedAtEnd = 0x02,
    StoppedAtError = 0x03,
    StoppedAtStart = 0x04,
    Starting = 0x08,
    Playing = 0x10,
    Stopping = 0x18,
    Pausing = 0x20,
    Paused = 0x28,
    PausedAtEnd = 0x30,
    Resuming = 0x38,
}

impl PlayerState {
    pub fn is_stopped(self) -> bool {
        matches!(
            self,
            Self::Stopped | Self::StoppedAtEnd | Self::StoppedAtError | Self::StoppedAtStart
        )
    }

    pub fn is_paused(self) -> bool {
        matches!(self, Self::Paused | Self::PausedAtEnd)
    }

    pub fn is_fading(self) -> bool {
        matches!(
            self,
            Self::Starting | Self::Stopping | Self::Pausing | Self::Resuming
        )
    }

    pub fn is_active(self) -> bool {
        !self.is_stopped() && !self.is_paused()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackState {
    pub id: AudioMsgId,
    pub state: PlayerState,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub frequency: u32,
    pub speed: f32,
}

impl TrackState {
    pub fn new(id: AudioMsgId, duration_ms: u64) -> Self {
        Self {
            id,
            state: PlayerState::Stopped,
            position_ms: 0,
            duration_ms,
            frequency: DEFAULT_FREQUENCY,
            speed: 1.0,
        }
    }

    pub fn progress(&self) -> f64 {
        if self.duration_ms == 0 {
            return 0.0;
        }
        (self.position_ms as f64 / self.duration_ms as f64).clamp(0.0, 1.0)
    }
}

/// Manages active audio track, playlist queue, volume, and playback speed.
#[derive(Debug, Default)]
pub struct AudioPlayer {
    current: Option<TrackState>,
    queue: Vec<TrackState>,
    volume: f32,
    speed: f32,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            current: None,
            queue: Vec::new(),
            volume: 1.0,
            speed: 1.0,
        }
    }

    pub fn play(&mut self, mut track: TrackState) {
        track.state = PlayerState::Playing;
        track.speed = self.speed;
        self.current = Some(track);
    }

    pub fn enqueue(&mut self, track: TrackState) {
        self.queue.push(track);
    }

    pub fn pause(&mut self) {
        if let Some(ref mut t) = self.current {
            if t.state == PlayerState::Playing {
                t.state = PlayerState::Paused;
            }
        }
    }

    pub fn resume(&mut self) {
        if let Some(ref mut t) = self.current {
            if t.state.is_paused() {
                t.state = PlayerState::Playing;
            }
        }
    }

    pub fn seek(&mut self, position_ms: u64) {
        if let Some(ref mut t) = self.current {
            t.position_ms = position_ms.min(t.duration_ms);
        }
    }

    pub fn set_speed(&mut self, speed: f32) {
        let clamped = speed.clamp(0.5, 2.5);
        self.speed = clamped;
        if let Some(ref mut t) = self.current {
            t.speed = clamped;
        }
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn current(&self) -> Option<&TrackState> {
        self.current.as_ref()
    }

    pub fn next_in_queue(&mut self) -> Option<&TrackState> {
        if let Some(next) = self.queue.first().cloned() {
            self.queue.remove(0);
            self.play(next);
            self.current.as_ref()
        } else {
            if let Some(ref mut t) = self.current {
                t.state = PlayerState::StoppedAtEnd;
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_playback_queue_and_seek() {
        let mut player = AudioPlayer::new();
        let voice1 = TrackState::new(
            AudioMsgId {
                audio_type: AudioType::Voice,
                document_id: 100,
                msg_id: 10,
            },
            10_000,
        );
        let voice2 = TrackState::new(
            AudioMsgId {
                audio_type: AudioType::Voice,
                document_id: 101,
                msg_id: 11,
            },
            5_000,
        );

        player.play(voice1);
        player.enqueue(voice2);
        player.set_speed(1.5);
        player.seek(5_000);

        let cur = player.current().unwrap();
        assert!(cur.state.is_active());
        assert_eq!(cur.speed, 1.5);
        assert!((cur.progress() - 0.5).abs() < 1e-6);

        player.pause();
        assert!(player.current().unwrap().state.is_paused());
        player.resume();
        assert!(player.current().unwrap().state.is_active());

        player.next_in_queue();
        assert_eq!(player.current().unwrap().id.msg_id, 11);
        assert_eq!(player.current().unwrap().speed, 1.5);

        assert!(player.next_in_queue().is_none());
        assert!(player.current().unwrap().state.is_stopped());
    }
}
