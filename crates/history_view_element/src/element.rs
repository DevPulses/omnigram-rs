use data_media_types::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HistoryContext {
    History,
    Replies,
    Pinned,
    AdminLog,
    ChatPreview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementChatMode {
    Default,
    Wide,
    Narrow,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ElementGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl ElementGeometry {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}
