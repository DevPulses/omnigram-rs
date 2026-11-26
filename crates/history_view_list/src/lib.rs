//! Virtualized message history list and chat widget layout.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/history/history_widget.{h,cpp}`
//! and `tdesktop/Telegram/SourceFiles/history/history_inner_widget.{h,cpp}`.

use history_view_element::MessageView;
use models_history::flags::MsgId;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PositionedMessage {
    pub msg_id: MsgId,
    pub top: i32,
    pub height: i32,
}

/// Virtualized scrollable history list managing message y-positions and viewport slicing.
#[derive(Debug, Default)]
pub struct HistoryListWidget {
    items: Vec<MessageView>,
    positions: Vec<PositionedMessage>,
    total_height: i32,
    scroll_top: i32,
    viewport_height: i32,
    item_spacing: i32,
}

impl HistoryListWidget {
    pub fn new(viewport_height: i32) -> Self {
        Self {
            items: Vec::new(),
            positions: Vec::new(),
            total_height: 0,
            scroll_top: 0,
            viewport_height: viewport_height.max(0),
            item_spacing: 4,
        }
    }

    pub fn push_message(&mut self, view: MessageView) {
        let top = self.total_height;
        let height = view.geometry.height.max(1);
        let msg_id = view.id;
        self.positions.push(PositionedMessage {
            msg_id,
            top,
            height,
        });
        self.total_height += height + self.item_spacing;
        self.items.push(view);
    }

    pub fn total_height(&self) -> i32 {
        self.total_height
    }

    pub fn scroll_top(&self) -> i32 {
        self.scroll_top
    }

    pub fn set_scroll_top(&mut self, y: i32) {
        let max_scroll = (self.total_height - self.viewport_height).max(0);
        self.scroll_top = y.clamp(0, max_scroll);
    }

    pub fn scroll_to_bottom(&mut self) {
        let max_scroll = (self.total_height - self.viewport_height).max(0);
        self.scroll_top = max_scroll;
    }

    pub fn scroll_to_msg_id(&mut self, msg_id: MsgId) -> bool {
        if let Some(pos) = self.positions.iter().find(|p| p.msg_id == msg_id) {
            self.set_scroll_top(pos.top);
            true
        } else {
            false
        }
    }

    pub fn visible_items(&self) -> Vec<&MessageView> {
        let view_top = self.scroll_top;
        let view_bottom = self.scroll_top + self.viewport_height;
        self.positions
            .iter()
            .enumerate()
            .filter(|(_, p)| p.top + p.height > view_top && p.top < view_bottom)
            .map(|(i, _)| &self.items[i])
            .collect()
    }

    pub fn hit_test(&self, y_in_viewport: i32) -> Option<MsgId> {
        let y_content = y_in_viewport + self.scroll_top;
        self.positions
            .iter()
            .find(|p| y_content >= p.top && y_content < p.top + p.height)
            .map(|p| p.msg_id)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use models_history::item::HistoryItem;
    use models_peers::PeerId;

    #[test]
    fn test_virtualized_history_list() {
        let mut list = HistoryListWidget::new(100);
        let peer = PeerId::from_raw(1000);

        for i in 1..=10 {
            let mut item = HistoryItem::new(i, peer, 1700000000 + i);
            item.text = format!("Message #{i}");
            let view = MessageView::new(&item, "12:00", 120, 24, 600);
            list.push_message(view);
        }

        assert_eq!(list.len(), 10);
        assert!(list.total_height() > 240);

        let visible_top = list.visible_items();
        assert!(!visible_top.is_empty());
        assert_eq!(visible_top[0].id, 1);

        list.scroll_to_bottom();
        let visible_bottom = list.visible_items();
        assert_eq!(visible_bottom.last().unwrap().id, 10);

        assert!(list.scroll_to_msg_id(5));
        let hit = list.hit_test(5);
        assert_eq!(hit, Some(5));
    }
}
