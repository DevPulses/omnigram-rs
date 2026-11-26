//! Dialog list with ordering, insert/update/remove.
//!
//! Ported from `dialogs/dialogs_list.h/.cpp`.

use std::collections::BTreeMap;

use crate::common::SortMode;
use crate::key::DialogsKey;

/// Numeric sort key used by `DialogsList`.
pub type SortKey = u64;

/// List update notification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositionChange {
    pub moved: bool,
    pub distance: i32,
}

/// A dialog row-like node with sort metadata.
#[derive(Debug, Clone)]
pub struct DialogRow {
    pub key: DialogsKey,
    pub sort_key: SortKey,
    pub height: i32,
    pub top: i32,
}

impl DialogRow {
    pub fn new(key: DialogsKey, sort_key: SortKey) -> Self {
        Self {
            key,
            sort_key,
            height: 0,
            top: 0,
        }
    }
}

/// List of dialogs in sorted order.
///
/// Simplified from tdesktop: keeps a stable order by `sort_key` then by key identity.
#[derive(Debug, Clone, Default)]
pub struct DialogsList {
    sort_mode: SortMode,
    rows: Vec<DialogRow>,
    by_key: BTreeMap<usize, usize>, // ptr key -> index
}

impl DialogsList {
    pub fn new(sort_mode: SortMode) -> Self {
        Self {
            sort_mode,
            rows: Vec::new(),
            by_key: BTreeMap::new(),
        }
    }

    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub fn len(&self) -> usize {
        self.rows.len()
    }

    pub fn rows(&self) -> &[DialogRow] {
        &self.rows
    }

    pub fn entry_at(&self, index: usize) -> Option<&DialogsKey> {
        self.rows.get(index).map(|r| &r.key)
    }

    pub fn find_row(&self, key: &DialogsKey) -> Option<&DialogRow> {
        let id = entry_ptr_id(key);
        self.by_key.get(&id).and_then(|i| self.rows.get(*i))
    }

    pub fn find_row_mut(&mut self, key: &DialogsKey) -> Option<&mut DialogRow> {
        let id = entry_ptr_id(key);
        let idx = *self.by_key.get(&id)?;
        self.rows.get_mut(idx)
    }

    /// Insert key into the list, returning position change if it existed.
    pub fn insert(&mut self, key: DialogsKey) -> PositionChange {
        let id = entry_ptr_id(&key);
        if self.by_key.contains_key(&id) {
            return PositionChange {
                moved: false,
                distance: 0,
            };
        }
        let sort_key = key.entry().sort_key_in_chat_list(self.sort_mode);
        let row = DialogRow::new(key, sort_key);
        let idx = self.sorted_insert_index_for(&row);
        self.rows.insert(idx, row);
        self.rebuild_index_from(idx);
        PositionChange {
            moved: true,
            distance: idx as i32,
        }
    }

    /// Reorder based on current entry sorting data.
    pub fn adjust_by_data(&mut self, key: &DialogsKey) -> Option<PositionChange> {
        let id = entry_ptr_id(key);
        let old_idx = *self.by_key.get(&id)?;
        let new_sort_key = self.rows[old_idx].key.entry().sort_key_in_chat_list(self.sort_mode);
        if self.rows[old_idx].sort_key == new_sort_key {
            return Some(PositionChange {
                moved: false,
                distance: 0,
            });
        }
        // Remove and reinsert
        let row = self.rows.remove(old_idx);
        self.rebuild_index_from(old_idx);
        let mut moved_row = row;
        moved_row.sort_key = new_sort_key;
        let new_idx = self.sorted_insert_index_for(&moved_row);
        self.rows.insert(new_idx, moved_row);
        self.rebuild_index_from(old_idx.min(new_idx));
        Some(PositionChange {
            moved: true,
            distance: (new_idx as i32) - (old_idx as i32),
        })
    }

    /// For name sort adjustments (simplified): re-sorts by name string.
    pub fn adjust_by_name(&mut self, key: &DialogsKey) -> Option<PositionChange> {
        let id = entry_ptr_id(key);
        let old_idx = *self.by_key.get(&id)?;
        let name = self.rows[old_idx].key.entry().chat_list_name().to_lowercase();
        // In name mode we compare names, else keep numeric sort key.
        let new_idx_target = if self.sort_mode == SortMode::Name {
            let mut candidate = 0usize;
            for (i, row) in self.rows.iter().enumerate() {
                if i == old_idx {
                    continue;
                }
                let other = row.key.entry().chat_list_name().to_lowercase();
                if name >= other {
                    candidate = i + 1;
                } else if i < old_idx {
                    candidate = i;
                }
            }
            candidate.min(self.rows.len().saturating_sub(1))
        } else {
            old_idx
        };
        if new_idx_target == old_idx {
            return Some(PositionChange {
                moved: false,
                distance: 0,
            });
        }
        let row = self.rows.remove(old_idx);
        let insert_idx = if new_idx_target > old_idx { new_idx_target } else { new_idx_target };
        self.rows.insert(insert_idx, row);
        self.rebuild_index_from(old_idx.min(insert_idx));
        Some(PositionChange {
            moved: true,
            distance: (insert_idx as i32) - (old_idx as i32),
        })
    }

    pub fn remove(&mut self, key: &DialogsKey, _replaced_by: Option<&DialogsKey>) -> bool {
        let id = entry_ptr_id(key);
        let Some(idx) = self.by_key.remove(&id) else {
            return false;
        };
        self.rows.remove(idx);
        self.rebuild_index_from(idx);
        true
    }

    pub fn row_at_y(&self, y: i32) -> Option<&DialogRow> {
        if y < 0 {
            return None;
        }
        let mut acc = 0i32;
        for row in &self.rows {
            acc += row.height.max(1);
            if y < acc {
                return Some(row);
            }
        }
        None
    }

    fn sorted_insert_index_for(&self, row: &DialogRow) -> usize {
        for (i, r) in self.rows.iter().enumerate() {
            if row.sort_key > r.sort_key {
                return i;
            }
            if row.sort_key == r.sort_key && row.key < r.key {
                return i;
            }
        }
        self.rows.len()
    }

    fn rebuild_index_from(&mut self, start: usize) {
        let start = start.min(self.rows.len());
        for i in start..self.rows.len() {
            let id = entry_ptr_id(&self.rows[i].key);
            self.by_key.insert(id, i);
        }
    }
}

fn entry_ptr_id(key: &DialogsKey) -> usize {
    // Use the underlying Arc pointer address as identity.
    key.entry().as_ref() as *const _ as *const () as usize
}
