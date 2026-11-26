//! Dialogs sidebar widget and inner scrollable list layout.
//!
//! Ported from `tdesktop/Telegram/SourceFiles/dialogs/dialogs_widget.{h,cpp}`
//! and `tdesktop/Telegram/SourceFiles/dialogs/dialogs_inner_widget.{h,cpp}`.

use std::ops::Range;

use dialogs_indexed_list::{DialogsKey, IndexedList};
use ui_dialog_row::DEFAULT_DIALOG_ROW;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogsLayout {
    Main,
    Child,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatSearchTab {
    Chats,
    Channels,
    Apps,
    Posts,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChosenRow {
    pub key: DialogsKey,
    pub index: usize,
    pub userpic_click: bool,
}

/// Inner scrollable list managing dialog row geometry, hit-testing, and selection.
pub struct DialogsInnerWidget {
    row_height: i32,
    scroll_top: i32,
    viewport_height: i32,
    selected_index: Option<usize>,
    active_key: Option<DialogsKey>,
}

impl Default for DialogsInnerWidget {
    fn default() -> Self {
        Self::new(600)
    }
}

impl DialogsInnerWidget {
    pub fn new(viewport_height: i32) -> Self {
        Self {
            row_height: DEFAULT_DIALOG_ROW.height,
            scroll_top: 0,
            viewport_height: viewport_height.max(0),
            selected_index: None,
            active_key: None,
        }
    }

    pub fn row_height(&self) -> i32 {
        self.row_height
    }

    pub fn content_height(&self, list: &IndexedList) -> i32 {
        (list.list().len() as i32) * self.row_height
    }

    pub fn set_scroll_top(&mut self, scroll_top: i32, list: &IndexedList) {
        let max_scroll = (self.content_height(list) - self.viewport_height).max(0);
        self.scroll_top = scroll_top.clamp(0, max_scroll);
    }

    pub fn visible_range(&self, list: &IndexedList) -> Range<usize> {
        let count = list.list().len();
        if count == 0 || self.row_height <= 0 {
            return 0..0;
        }
        let start = ((self.scroll_top / self.row_height).max(0) as usize).min(count);
        let end = (((self.scroll_top + self.viewport_height + self.row_height - 1)
            / self.row_height)
            .max(0) as usize)
            .min(count);
        start..end
    }

    pub fn row_at_y(&self, y_in_content: i32, list: &IndexedList) -> Option<usize> {
        if y_in_content < 0 || self.row_height <= 0 {
            return None;
        }
        let idx = (y_in_content / self.row_height) as usize;
        if idx < list.list().len() {
            Some(idx)
        } else {
            None
        }
    }

    pub fn select_at_y(&mut self, y_in_viewport: i32, list: &IndexedList) -> Option<ChosenRow> {
        let y_content = y_in_viewport + self.scroll_top;
        let idx = self.row_at_y(y_content, list)?;
        let row = list.list().rows().get(idx)?;
        let key = row.key.clone();
        self.selected_index = Some(idx);
        self.active_key = Some(key.clone());
        Some(ChosenRow {
            key,
            index: idx,
            userpic_click: false,
        })
    }

    pub fn active_key(&self) -> Option<&DialogsKey> {
        self.active_key.as_ref()
    }
}

/// Top-level dialogs widget managing search query, layout mode, and inner list.
pub struct DialogsWidget {
    pub layout: DialogsLayout,
    pub search_query: String,
    pub search_tab: ChatSearchTab,
    pub inner: DialogsInnerWidget,
}

impl DialogsWidget {
    pub fn new(layout: DialogsLayout, viewport_height: i32) -> Self {
        Self {
            layout,
            search_query: String::new(),
            search_tab: ChatSearchTab::Chats,
            inner: DialogsInnerWidget::new(viewport_height),
        }
    }

    pub fn filter_entries(&self, list: &IndexedList) -> Vec<DialogsKey> {
        if self.search_query.trim().is_empty() {
            list.list().rows().iter().map(|r| r.key.clone()).collect()
        } else {
            list.peer_search_results(&self.search_query)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dialogs_indexed_list::{DialogsEntry, DialogsEntryState, SortMode};
    use models_peers::{PeerId, UserId};
    use std::sync::Arc;

    fn make_entry(id: u64, name: &str, date: i32) -> DialogsKey {
        let peer = PeerId::from_user(UserId::new(id));
        let state = DialogsEntryState::new_history(id, peer, name);
        let entry = Arc::new(DialogsEntry::new(state));
        entry.set_chat_list_time_id(date);
        DialogsKey::new(entry)
    }

    #[test]
    fn test_visible_range_and_hit_testing() {
        let mut list = IndexedList::new(SortMode::Date, 0);
        for i in 1..=20 {
            list.add(make_entry(i, &format!("User {i}"), 1000 + i as i32));
        }

        let mut widget = DialogsWidget::new(DialogsLayout::Main, 200);
        assert_eq!(widget.inner.content_height(&list), 20 * DEFAULT_DIALOG_ROW.height);
        assert_eq!(widget.inner.visible_range(&list), 0..4);

        widget.inner.set_scroll_top(DEFAULT_DIALOG_ROW.height * 2, &list);
        assert_eq!(widget.inner.visible_range(&list), 2..6);

        let chosen = widget.inner.select_at_y(10, &list).unwrap();
        assert_eq!(chosen.index, 2);
        assert!(widget.inner.active_key().is_some());
    }

    #[test]
    fn test_search_filtering() {
        let mut list = IndexedList::new(SortMode::Date, 0);
        list.add(make_entry(1, "Alice Rust", 100));
        list.add(make_entry(2, "Bob Cpp", 200));

        let mut widget = DialogsWidget::new(DialogsLayout::Main, 400);
        assert_eq!(widget.filter_entries(&list).len(), 2);

        widget.search_query = "alice".to_string();
        let filtered = widget.filter_entries(&list);
        assert_eq!(filtered.len(), 1);
    }
}
