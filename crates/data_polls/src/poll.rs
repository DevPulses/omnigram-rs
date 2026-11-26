//! PollData, PollAnswer, and voting logic matching Telegram Desktop.
//!
//! Source: `tdesktop/Telegram/SourceFiles/data/data_poll.{h,cpp}`.

use std::fmt;

use data_media_types::{DocumentId, PhotoId};
use models_peers::PeerId;

pub type PollId = u64;

bitflags::bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct PollFlags: u32 {
        const CLOSED                 = 0x001;
        const PUBLIC_VOTES           = 0x002;
        const MULTI_CHOICE           = 0x004;
        const QUIZ                   = 0x008;
        const SHUFFLE_ANSWERS        = 0x010;
        const REVOTING_DISABLED      = 0x020;
        const OPEN_ANSWERS           = 0x040;
        const HIDE_RESULTS_UNTIL_CLOSE = 0x080;
        const CREATOR                = 0x100;
        const SUBSCRIBERS_ONLY       = 0x200;
        const CAN_VIEW_STATS         = 0x400;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VoteRestriction {
    #[default]
    None,
    SubscribersOnly,
    SubscribersJoinedTooRecently,
    Countries,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PollMedia {
    pub photo_id: Option<PhotoId>,
    pub document_id: Option<DocumentId>,
    pub url: Option<String>,
}

impl PollMedia {
    pub fn is_empty(&self) -> bool {
        self.photo_id.is_none() && self.document_id.is_none() && self.url.is_none()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollAnswer {
    pub text: String,
    pub option: Vec<u8>,
    pub media: PollMedia,
    pub recent_voters: Vec<PeerId>,
    pub added_by: Option<PeerId>,
    pub added_date: i32,
    pub votes: i32,
    pub chosen: bool,
    pub correct: bool,
}

impl PollAnswer {
    pub fn new(text: impl Into<String>, option: Vec<u8>) -> Self {
        Self {
            text: text.into(),
            option,
            media: PollMedia::default(),
            recent_voters: Vec::new(),
            added_by: None,
            added_date: 0,
            votes: 0,
            chosen: false,
            correct: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PollError {
    PollClosed,
    RevotingDisabled,
    MultipleChoicesNotAllowed,
    EmptyVote,
    UnknownOption,
}

impl fmt::Display for PollError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PollClosed => write!(f, "Poll is closed"),
            Self::RevotingDisabled => write!(f, "Revoting is disabled for this poll"),
            Self::MultipleChoicesNotAllowed => write!(f, "Multiple choices are not allowed"),
            Self::EmptyVote => write!(f, "At least one option must be selected"),
            Self::UnknownOption => write!(f, "Unknown poll option"),
        }
    }
}

impl std::error::Error for PollError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PollData {
    pub id: PollId,
    pub question: String,
    pub answers: Vec<PollAnswer>,
    pub recent_voters: Vec<PeerId>,
    pub sending_votes: Vec<Vec<u8>>,
    pub solution: String,
    pub attached_media: PollMedia,
    pub solution_media: PollMedia,
    pub close_period: i32,
    pub close_date: i32,
    pub total_voters: i32,
    pub version: i32,
    pub flags: PollFlags,
    pub vote_restriction: VoteRestriction,
}

impl PollData {
    pub fn new(id: PollId, question: impl Into<String>, flags: PollFlags) -> Self {
        Self {
            id,
            question: question.into(),
            answers: Vec::new(),
            recent_voters: Vec::new(),
            sending_votes: Vec::new(),
            solution: String::new(),
            attached_media: PollMedia::default(),
            solution_media: PollMedia::default(),
            close_period: 0,
            close_date: 0,
            total_voters: 0,
            version: 0,
            flags,
            vote_restriction: VoteRestriction::None,
        }
    }

    pub fn voted(&self) -> bool {
        self.answers.iter().any(|a| a.chosen)
    }

    pub fn closed(&self) -> bool {
        self.flags.contains(PollFlags::CLOSED)
    }

    pub fn public_votes(&self) -> bool {
        self.flags.contains(PollFlags::PUBLIC_VOTES)
    }

    pub fn multi_choice(&self) -> bool {
        self.flags.contains(PollFlags::MULTI_CHOICE)
    }

    pub fn quiz(&self) -> bool {
        self.flags.contains(PollFlags::QUIZ)
    }

    pub fn revoting_disabled(&self) -> bool {
        self.flags.contains(PollFlags::REVOTING_DISABLED) || self.quiz()
    }

    pub fn hide_results_until_close(&self) -> bool {
        self.flags.contains(PollFlags::HIDE_RESULTS_UNTIL_CLOSE)
    }

    pub fn answer_by_option(&self, option: &[u8]) -> Option<&PollAnswer> {
        self.answers.iter().find(|a| a.option.as_slice() == option)
    }

    pub fn answer_by_option_mut(&mut self, option: &[u8]) -> Option<&mut PollAnswer> {
        self.answers
            .iter_mut()
            .find(|a| a.option.as_slice() == option)
    }

    pub fn close_by_timer(&mut self, now_unix: i32) -> bool {
        if !self.closed() && self.close_date > 0 && now_unix >= self.close_date {
            self.flags.insert(PollFlags::CLOSED);
            return true;
        }
        false
    }

    /// Applies a local vote selection, validating poll flags and updating counts.
    pub fn vote(&mut self, selected_options: &[Vec<u8>]) -> Result<(), PollError> {
        if self.closed() {
            return Err(PollError::PollClosed);
        }
        if selected_options.is_empty() {
            return Err(PollError::EmptyVote);
        }
        if !self.multi_choice() && selected_options.len() > 1 {
            return Err(PollError::MultipleChoicesNotAllowed);
        }
        if self.voted() && self.revoting_disabled() {
            return Err(PollError::RevotingDisabled);
        }
        for opt in selected_options {
            if self.answer_by_option(opt).is_none() {
                return Err(PollError::UnknownOption);
            }
        }

        let was_voted = self.voted();
        for ans in &mut self.answers {
            if ans.chosen {
                ans.chosen = false;
                ans.votes = (ans.votes - 1).max(0);
            }
        }
        for opt in selected_options {
            if let Some(ans) = self.answer_by_option_mut(opt) {
                ans.chosen = true;
                ans.votes += 1;
            }
        }
        if !was_voted {
            self.total_voters += 1;
        }
        self.sending_votes = selected_options.to_vec();
        Ok(())
    }

    /// Retracts the user's votes if revoting is allowed.
    pub fn retract_vote(&mut self) -> Result<(), PollError> {
        if self.closed() {
            return Err(PollError::PollClosed);
        }
        if self.revoting_disabled() {
            return Err(PollError::RevotingDisabled);
        }
        if self.voted() {
            for ans in &mut self.answers {
                if ans.chosen {
                    ans.chosen = false;
                    ans.votes = (ans.votes - 1).max(0);
                }
            }
            self.total_voters = (self.total_voters - 1).max(0);
            self.sending_votes.clear();
        }
        Ok(())
    }

    /// Computes integer percentages (0..=100) for each answer.
    pub fn percentages(&self) -> Vec<i32> {
        if self.total_voters <= 0 {
            return vec![0; self.answers.len()];
        }
        self.answers
            .iter()
            .map(|a| ((a.votes as f64 / self.total_voters as f64) * 100.0).round() as i32)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_choice_and_retract() {
        let mut poll = PollData::new(1, "Best language?", PollFlags::PUBLIC_VOTES);
        poll.answers.push(PollAnswer::new("Rust", b"0".to_vec()));
        poll.answers.push(PollAnswer::new("C++", b"1".to_vec()));

        assert_eq!(
            poll.vote(&[b"0".to_vec(), b"1".to_vec()]),
            Err(PollError::MultipleChoicesNotAllowed)
        );

        poll.vote(&[b"0".to_vec()]).unwrap();
        assert!(poll.voted());
        assert_eq!(poll.total_voters, 1);
        assert_eq!(poll.percentages(), vec![100, 0]);

        poll.retract_vote().unwrap();
        assert!(!poll.voted());
        assert_eq!(poll.total_voters, 0);
    }

    #[test]
    fn test_quiz_prevents_revote_and_timer_closes() {
        let mut quiz = PollData::new(2, "2 + 2 = ?", PollFlags::QUIZ);
        let mut a1 = PollAnswer::new("3", b"a".to_vec());
        a1.correct = false;
        let mut a2 = PollAnswer::new("4", b"b".to_vec());
        a2.correct = true;
        quiz.answers.push(a1);
        quiz.answers.push(a2);
        quiz.close_date = 1700001000;

        quiz.vote(&[b"b".to_vec()]).unwrap();
        assert_eq!(quiz.vote(&[b"a".to_vec()]), Err(PollError::RevotingDisabled));

        assert!(!quiz.close_by_timer(1700000500));
        assert!(quiz.close_by_timer(1700001000));
        assert!(quiz.closed());
    }

    #[test]
    fn test_multi_choice_poll() {
        let mut poll = PollData::new(3, "Platforms?", PollFlags::MULTI_CHOICE);
        poll.answers.push(PollAnswer::new("Linux", b"l".to_vec()));
        poll.answers.push(PollAnswer::new("macOS", b"m".to_vec()));
        poll.answers.push(PollAnswer::new("Windows", b"w".to_vec()));

        poll.vote(&[b"l".to_vec(), b"m".to_vec()]).unwrap();
        assert_eq!(poll.total_voters, 1);
        assert!(poll.answers[0].chosen);
        assert!(poll.answers[1].chosen);
        assert!(!poll.answers[2].chosen);
    }
}
