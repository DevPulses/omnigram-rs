//! Polls and quizzes model and voting state transitions.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/data/data_poll.{h,cpp}`
//! and `tdesktop/Telegram/SourceFiles/data/data_poll_messages.{h,cpp}`.

pub mod poll;

pub use poll::{
    PollAnswer, PollData, PollError, PollFlags, PollId, PollMedia, VoteRestriction,
};
