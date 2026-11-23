//! Message bubble layout, metrics, and geometry matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/history/view/history_view_message.{h,cpp}` and `chat.style`).

use models_history::flags::MsgId;
use models_peers::id::PeerId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Margins {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Margins {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self { left, top, right, bottom }
    }
}

// Styling constants ported from `chat.style`
pub const MSG_MAX_WIDTH: i32 = 430;
pub const MSG_MIN_WIDTH: i32 = 160;
pub const MSG_PHOTO_SIZE: i32 = 33;
pub const MSG_PHOTO_SKIP: i32 = 40;
pub const MSG_PADDING: Margins = Margins::new(11, 8, 11, 8);
pub const MSG_MARGIN: Margins = Margins::new(16, 6, 56, 2);
pub const MSG_MARGIN_TOP_ATTACHED: i32 = 0;
pub const MSG_SHADOW: i32 = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BubbleGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub inner_width: i32,
    pub inner_height: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MessageBubbleItem {
    pub id: MsgId,
    pub from: PeerId,
    pub text: String,
    pub date: i32,
    pub out: bool,
    pub has_from_photo: bool,
    pub attached_to_next: bool,
}

impl MessageBubbleItem {
    pub fn new(id: MsgId, from: PeerId, text: impl Into<String>, out: bool) -> Self {
        Self {
            id,
            from,
            text: text.into(),
            date: 0,
            out,
            has_from_photo: !out,
            attached_to_next: false,
        }
    }

    /// Calculate bubble layout bounding box within container width
    pub fn count_geometry(&self, text_width: i32, text_height: i32, container_width: i32) -> BubbleGeometry {
        let content_w = std::cmp::max(text_width, MSG_MIN_WIDTH - MSG_PADDING.left - MSG_PADDING.right);
        let bubble_w = std::cmp::min(content_w + MSG_PADDING.left + MSG_PADDING.right, MSG_MAX_WIDTH);
        let bubble_h = text_height + MSG_PADDING.top + MSG_PADDING.bottom;

        let x = if self.out {
            // Right-aligned for outgoing messages
            container_width - bubble_w - MSG_MARGIN.right
        } else {
            // Left-aligned for incoming messages, skipping user photo if present
            let left_skip = if self.has_from_photo {
                MSG_MARGIN.left + MSG_PHOTO_SKIP
            } else {
                MSG_MARGIN.left
            };
            left_skip
        };

        let y = MSG_MARGIN.top;

        BubbleGeometry {
            x,
            y,
            width: bubble_w,
            height: bubble_h,
            inner_width: bubble_w - MSG_PADDING.left - MSG_PADDING.right,
            inner_height: bubble_h - MSG_PADDING.top - MSG_PADDING.bottom,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_peers::id::UserId;

    #[test]
    fn test_chat_style_constants() {
        assert_eq!(MSG_MAX_WIDTH, 430);
        assert_eq!(MSG_MIN_WIDTH, 160);
        assert_eq!(MSG_PHOTO_SIZE, 33);
        assert_eq!(MSG_PHOTO_SKIP, 40);
        assert_eq!(MSG_PADDING.left, 11);
        assert_eq!(MSG_PADDING.top, 8);
    }

    #[test]
    fn test_incoming_bubble_geometry() {
        let peer = PeerId::from_user(UserId::new(100));
        let item = MessageBubbleItem::new(1, peer, "Hello", false);
        let geom = item.count_geometry(100, 20, 800);

        // Min width constraint: content_w clamped to 160 - 22 = 138 -> bubble_w = 160
        assert_eq!(geom.width, MSG_MIN_WIDTH);
        assert_eq!(geom.height, 20 + 8 + 8);
        assert_eq!(geom.x, MSG_MARGIN.left + MSG_PHOTO_SKIP);
    }

    #[test]
    fn test_outgoing_bubble_geometry() {
        let peer = PeerId::from_user(UserId::new(100));
        let item = MessageBubbleItem::new(2, peer, "Long text message here...", true);
        let geom = item.count_geometry(200, 30, 800);

        assert_eq!(geom.width, 200 + 22);
        assert_eq!(geom.height, 30 + 16);
        assert_eq!(geom.x, 800 - (200 + 22) - MSG_MARGIN.right);
    }
}
