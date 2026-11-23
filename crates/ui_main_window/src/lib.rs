//! Main window layout, geometry constraints, and position management matching Telegram Desktop
//! (`tdesktop/Telegram/SourceFiles/window/main_window.{h,cpp}` and `window.style`).

use core_settings::window_position::WindowPosition;

// Constants from window.style and main_window.cpp
pub const WINDOW_MIN_WIDTH: i32 = 380;
pub const WINDOW_MIN_HEIGHT: i32 = 480;
pub const WINDOW_DEFAULT_WIDTH: i32 = 800;
pub const WINDOW_DEFAULT_HEIGHT: i64 = 600;

pub const COLUMN_MIN_WIDTH_LEFT: i32 = 260;
pub const COLUMN_MAX_WIDTH_LEFT: i32 = 540;
pub const COLUMN_MIN_WIDTH_MAIN: i32 = 380;
pub const COLUMN_MIN_WIDTH_THIRD: i32 = 292;
pub const COLUMN_MAX_WIDTH_THIRD: i32 = 392;

pub const ADAPTIVE_CHAT_WIDE_WIDTH: i32 = 880;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn is_empty(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
    FullScreen,
}

impl Default for WindowState {
    fn default() -> Self {
        Self::Normal
    }
}

/// Window geometry constraints and column allocation calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowLayout {
    pub window_rect: Rect,
    pub state: WindowState,
    pub left_column_width: i32,
    pub third_column_visible: bool,
    pub third_column_width: i32,
}

impl Default for WindowLayout {
    fn default() -> Self {
        Self {
            window_rect: Rect::new(0, 0, WINDOW_DEFAULT_WIDTH, 600),
            state: WindowState::Normal,
            left_column_width: COLUMN_MIN_WIDTH_LEFT,
            third_column_visible: false,
            third_column_width: 0,
        }
    }
}

impl WindowLayout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn min_width(&self) -> i32 {
        let mut min_w = COLUMN_MIN_WIDTH_LEFT + COLUMN_MIN_WIDTH_MAIN;
        if self.third_column_visible {
            min_w += COLUMN_MIN_WIDTH_THIRD;
        }
        std::cmp::max(WINDOW_MIN_WIDTH, min_w)
    }

    pub fn min_height(&self) -> i32 {
        WINDOW_MIN_HEIGHT
    }

    pub fn main_column_width(&self) -> i32 {
        let mut avail = self.window_rect.width - self.left_column_width;
        if self.third_column_visible {
            avail -= self.third_column_width;
        }
        std::cmp::max(0, avail)
    }

    pub fn is_wide_chat(&self) -> bool {
        self.main_column_width() >= ADAPTIVE_CHAT_WIDE_WIDTH
    }

    /// Try to extend width by `add_to_width`, bounded by screen / max desktop width.
    pub fn try_to_extend_width_by(&mut self, add_to_width: i32, screen_width: i32) -> i32 {
        let current = self.window_rect.width;
        let target = current + add_to_width;
        let clamped = if screen_width > 0 {
            std::cmp::min(target, screen_width)
        } else {
            target
        };
        let delta = clamped - current;
        if delta > 0 {
            self.window_rect.width += delta;
        }
        delta
    }

    /// Apply saved position from Core::WindowPosition
    pub fn apply_saved_position(&mut self, pos: &WindowPosition) {
        if pos.w >= WINDOW_MIN_WIDTH && pos.h >= WINDOW_MIN_HEIGHT {
            self.window_rect = Rect::new(pos.x, pos.y, pos.w, pos.h);
        }
        if pos.maximized != 0 {
            self.state = WindowState::Maximized;
        } else {
            self.state = WindowState::Normal;
        }
    }

    /// Export position to Core::WindowPosition for saving
    pub fn count_position_for_save(&self) -> WindowPosition {
        WindowPosition {
            x: self.window_rect.x,
            y: self.window_rect.y,
            w: self.window_rect.width,
            h: self.window_rect.height,
            scale: 100, // 100% default scale
            maximized: if self.state == WindowState::Maximized { 1 } else { 0 },
            moncrc: 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_layout() {
        let layout = WindowLayout::default();
        assert_eq!(layout.window_rect.width, 800);
        assert_eq!(layout.left_column_width, COLUMN_MIN_WIDTH_LEFT);
        assert_eq!(layout.main_column_width(), 800 - 260);
        assert!(!layout.is_wide_chat());
    }

    #[test]
    fn test_min_constraints() {
        let mut layout = WindowLayout::default();
        assert_eq!(layout.min_width(), 260 + 380);
        assert_eq!(layout.min_height(), 480);

        layout.third_column_visible = true;
        layout.third_column_width = 300;
        assert_eq!(layout.min_width(), 260 + 380 + 292);
    }

    #[test]
    fn test_saved_position_roundtrip() {
        let mut layout = WindowLayout::default();
        let pos = WindowPosition {
            x: 100,
            y: 150,
            w: 1024,
            h: 768,
            scale: 100,
            maximized: 1,
            moncrc: 1,
        };
        layout.apply_saved_position(&pos);
        assert_eq!(layout.window_rect.width, 1024);
        assert_eq!(layout.window_rect.height, 768);
        assert_eq!(layout.state, WindowState::Maximized);

        let exported = layout.count_position_for_save();
        assert_eq!(exported.x, 100);
        assert_eq!(exported.y, 150);
        assert_eq!(exported.w, 1024);
        assert_eq!(exported.h, 768);
        assert_eq!(exported.maximized, 1);
    }

    #[test]
    fn test_extend_width() {
        let mut layout = WindowLayout::default();
        let extended = layout.try_to_extend_width_by(200, 1920);
        assert_eq!(extended, 200);
        assert_eq!(layout.window_rect.width, 1000);

        // Limit to screen
        let extended2 = layout.try_to_extend_width_by(1500, 1920);
        assert_eq!(extended2, 920);
        assert_eq!(layout.window_rect.width, 1920);
    }
}
