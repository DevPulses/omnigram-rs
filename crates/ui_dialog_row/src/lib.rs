//! Dialog row styling, layout metrics, and paint context matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/dialogs/ui/dialogs_layout.{h,cpp}` and `dialogs.style`).

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// Metrics and styling for a dialog row.
/// Exact values ported from `tdesktop/Telegram/SourceFiles/dialogs/dialogs.style`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DialogRowStyle {
    pub height: i32,
    pub padding: Margins,
    pub photo_size: i32,
    pub photo_position: Point,
    pub name_position: Point,
    pub text_position: Point,
}

pub const DEFAULT_DIALOG_ROW: DialogRowStyle = DialogRowStyle {
    height: 62,
    padding: Margins::new(10, 8, 10, 8),
    photo_size: 46,
    photo_position: Point::new(10, 8),
    name_position: Point::new(68, 10),
    text_position: Point::new(68, 34),
};

pub const TAGGED_DIALOG_ROW: DialogRowStyle = DialogRowStyle {
    height: 72,
    padding: Margins::new(10, 8, 10, 8),
    photo_size: 46,
    photo_position: Point::new(10, 8),
    name_position: Point::new(68, 10),
    text_position: Point::new(68, 30),
};

pub const FORUM_DIALOG_ROW: DialogRowStyle = DialogRowStyle {
    height: 80,
    padding: Margins::new(10, 8, 10, 8),
    photo_size: 46,
    photo_position: Point::new(10, 8),
    name_position: Point::new(68, 10),
    text_position: Point::new(68, 32),
};

/// Narrow dialog list width:
/// `st::defaultDialogRow.padding.left() + st::defaultDialogRow.photoSize + st::defaultDialogRow.padding.left()`
pub fn narrow_width(st: &DialogRowStyle) -> i32 {
    st.padding.left + st.photo_size + st.padding.left
}

/// Dialog row view data model for rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogRowData {
    pub peer_id: PeerId,
    pub title: String,
    pub text: String,
    pub date_text: String,
    pub unread_count: i32,
    pub unread_muted: bool,
    pub is_pinned: bool,
    pub is_selected: bool,
    pub is_active: bool,
}

impl DialogRowData {
    pub fn new(peer_id: PeerId, title: impl Into<String>) -> Self {
        Self {
            peer_id,
            title: title.into(),
            text: String::new(),
            date_text: String::new(),
            unread_count: 0,
            unread_muted: false,
            is_pinned: false,
            is_selected: false,
            is_active: false,
        }
    }
}

/// Paint context for drawing a dialog row
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaintContext {
    pub style: DialogRowStyle,
    pub width: i32,
    pub active: bool,
    pub selected: bool,
    pub narrow: bool,
    pub display_unread_info: bool,
}

impl Default for PaintContext {
    fn default() -> Self {
        Self {
            style: DEFAULT_DIALOG_ROW,
            width: 380,
            active: false,
            selected: false,
            narrow: false,
            display_unread_info: true,
        }
    }
}

impl PaintContext {
    pub fn new(width: i32) -> Self {
        let narrow = width <= narrow_width(&DEFAULT_DIALOG_ROW);
        Self {
            width,
            narrow,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_peers::id::UserId;

    #[test]
    fn test_dialog_row_metrics() {
        assert_eq!(DEFAULT_DIALOG_ROW.height, 62);
        assert_eq!(DEFAULT_DIALOG_ROW.photo_size, 46);
        assert_eq!(DEFAULT_DIALOG_ROW.padding.left, 10);
        assert_eq!(DEFAULT_DIALOG_ROW.padding.top, 8);

        // Narrow width = 10 + 46 + 10 = 66
        assert_eq!(narrow_width(&DEFAULT_DIALOG_ROW), 66);
    }

    #[test]
    fn test_paint_context_narrow_detection() {
        let ctx_normal = PaintContext::new(380);
        assert!(!ctx_normal.narrow);

        let ctx_narrow = PaintContext::new(66);
        assert!(ctx_narrow.narrow);
    }

    #[test]
    fn test_row_data() {
        let peer = PeerId::from_user(UserId::new(999));
        let mut row = DialogRowData::new(peer, "Test Chat");
        row.unread_count = 5;
        row.is_pinned = true;

        assert_eq!(row.title, "Test Chat");
        assert_eq!(row.unread_count, 5);
        assert!(row.is_pinned);
    }
}
