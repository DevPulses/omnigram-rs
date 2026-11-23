//! Telegram Updates dispatcher and PTS sequence manager matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/api/api_updates.{h,cpp}` and `data/data_pts_waiter.{h,cpp}`).

use std::collections::BTreeMap;
use models_peers::id::ChannelId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PtsCheckResult {
    Applied,
    GapDetected { expected_pts: i32, received_pts: i32 },
    Outdated,
}

/// PTS waiter state for tracking PTS gaps in global and channel updates
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PtsWaiter {
    pts: i32,
    requesting_difference: bool,
    channel_pts: BTreeMap<ChannelId, i32>,
}

impl Default for PtsWaiter {
    fn default() -> Self {
        Self {
            pts: 0,
            requesting_difference: false,
            channel_pts: BTreeMap::new(),
        }
    }
}

impl PtsWaiter {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn pts(&self) -> i32 {
        self.pts
    }

    pub fn set_pts(&mut self, pts: i32) {
        self.pts = pts;
        self.requesting_difference = false;
    }

    pub fn requesting_difference(&self) -> bool {
        self.requesting_difference
    }

    pub fn set_requesting_difference(&mut self, requesting: bool) {
        self.requesting_difference = requesting;
    }

    /// Check and advance global PTS
    /// If pts == self.pts + pts_count -> no gap, advance
    pub fn check_and_advance(&mut self, pts: i32, pts_count: i32) -> PtsCheckResult {
        if self.pts == 0 {
            // First update initializes PTS
            self.pts = pts;
            return PtsCheckResult::Applied;
        }

        let expected_pts = self.pts + pts_count;
        if pts == expected_pts {
            self.pts = pts;
            PtsCheckResult::Applied
        } else if pts > expected_pts {
            self.requesting_difference = true;
            PtsCheckResult::GapDetected {
                expected_pts,
                received_pts: pts,
            }
        } else {
            PtsCheckResult::Outdated
        }
    }

    /// Channel specific PTS handling
    pub fn check_channel_pts(&mut self, channel: ChannelId, pts: i32, pts_count: i32) -> PtsCheckResult {
        let current = self.channel_pts.get(&channel).copied().unwrap_or(0);
        if current == 0 {
            self.channel_pts.insert(channel, pts);
            return PtsCheckResult::Applied;
        }

        let expected_pts = current + pts_count;
        if pts == expected_pts {
            self.channel_pts.insert(channel, pts);
            PtsCheckResult::Applied
        } else if pts > expected_pts {
            PtsCheckResult::GapDetected {
                expected_pts,
                received_pts: pts,
            }
        } else {
            PtsCheckResult::Outdated
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pts_advance_and_gap() {
        let mut waiter = PtsWaiter::new();
        // Initial PTS
        assert_eq!(waiter.check_and_advance(100, 1), PtsCheckResult::Applied);
        assert_eq!(waiter.pts(), 100);

        // Next consecutive update (pts 101, count 1)
        assert_eq!(waiter.check_and_advance(101, 1), PtsCheckResult::Applied);
        assert_eq!(waiter.pts(), 101);

        // Multi-event update (pts 105, count 4)
        assert_eq!(waiter.check_and_advance(105, 4), PtsCheckResult::Applied);
        assert_eq!(waiter.pts(), 105);

        // Gap: expected 106, got 110
        assert_eq!(
            waiter.check_and_advance(110, 1),
            PtsCheckResult::GapDetected {
                expected_pts: 106,
                received_pts: 110
            }
        );
        assert!(waiter.requesting_difference());

        // Outdated: got 104 when at 105
        assert_eq!(waiter.check_and_advance(104, 1), PtsCheckResult::Outdated);
    }

    #[test]
    fn test_channel_pts() {
        let mut waiter = PtsWaiter::new();
        let channel = ChannelId::new(42);

        assert_eq!(waiter.check_channel_pts(channel, 50, 1), PtsCheckResult::Applied);
        assert_eq!(waiter.check_channel_pts(channel, 52, 2), PtsCheckResult::Applied);
        assert_eq!(
            waiter.check_channel_pts(channel, 60, 1),
            PtsCheckResult::GapDetected {
                expected_pts: 53,
                received_pts: 60
            }
        );
    }
}
