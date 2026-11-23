//! Chat compose input field controls, metrics, and state matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/history/view/controls/history_view_compose_controls.{h,cpp}`
//! and `chat_helpers.style`).

// Constants ported from chat_helpers.style
pub const HISTORY_COMPOSE_FIELD_HEIGHT_MIN: i32 = 36;
pub const HISTORY_COMPOSE_FIELD_HEIGHT_MAX: i32 = 72;
pub const HISTORY_COMPOSE_FIELD_MAX_HEIGHT: i32 = 224;
pub const HISTORY_COMPOSE_FIELD_FADE_HEIGHT: i32 = 6;
pub const HISTORY_SEND_RIGHT: i32 = 2;
pub const HISTORY_SEND_PADDING: i32 = 9;

pub const SEND_BUTTON_SIZE: i32 = 36;
pub const ATTACH_BUTTON_SIZE: i32 = 36;
pub const EMOJI_BUTTON_SIZE: i32 = 36;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendButtonState {
    Send,
    RecordVoice,
    RecordRoundVideo,
    EditSave,
}

impl Default for SendButtonState {
    fn default() -> Self {
        Self::RecordVoice
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatInputState {
    pub text: String,
    pub cursor_pos: usize,
    pub is_recording: bool,
    pub is_editing: bool,
    pub is_empty: bool,
    pub field_height: i32,
}

impl Default for ChatInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            cursor_pos: 0,
            is_recording: false,
            is_editing: false,
            is_empty: true,
            field_height: HISTORY_COMPOSE_FIELD_HEIGHT_MIN,
        }
    }
}

impl ChatInputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        self.text = text.into();
        self.is_empty = self.text.trim().is_empty();
        self.cursor_pos = self.text.len();
    }

    pub fn send_button_state(&self) -> SendButtonState {
        if self.is_editing {
            SendButtonState::EditSave
        } else if !self.is_empty {
            SendButtonState::Send
        } else {
            SendButtonState::RecordVoice
        }
    }

    /// Update dynamic height based on text lines and content height
    pub fn update_height(&mut self, content_height: i32) {
        let clamped = content_height.clamp(
            HISTORY_COMPOSE_FIELD_HEIGHT_MIN,
            HISTORY_COMPOSE_FIELD_MAX_HEIGHT,
        );
        self.field_height = clamped;
    }

    /// Clear text after message submission
    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_pos = 0;
        self.is_empty = true;
        self.is_editing = false;
        self.field_height = HISTORY_COMPOSE_FIELD_HEIGHT_MIN;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state() {
        let state = ChatInputState::new();
        assert!(state.is_empty);
        assert_eq!(state.field_height, HISTORY_COMPOSE_FIELD_HEIGHT_MIN);
        // When empty, Telegram Desktop displays Voice Record button
        assert_eq!(state.send_button_state(), SendButtonState::RecordVoice);
    }

    #[test]
    fn test_text_and_send_button_switch() {
        let mut state = ChatInputState::new();
        state.set_text("Hello");
        assert!(!state.is_empty);
        // When text is entered, it changes to Send button
        assert_eq!(state.send_button_state(), SendButtonState::Send);

        state.is_editing = true;
        assert_eq!(state.send_button_state(), SendButtonState::EditSave);

        state.clear();
        assert!(state.is_empty);
        assert_eq!(state.send_button_state(), SendButtonState::RecordVoice);
    }

    #[test]
    fn test_dynamic_height_clamping() {
        let mut state = ChatInputState::new();
        state.update_height(20);
        assert_eq!(state.field_height, HISTORY_COMPOSE_FIELD_HEIGHT_MIN);

        state.update_height(100);
        assert_eq!(state.field_height, 100);

        state.update_height(500);
        assert_eq!(state.field_height, HISTORY_COMPOSE_FIELD_MAX_HEIGHT);
    }
}
