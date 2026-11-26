//! Indexed dialog list (by first letter + date/name buckets).
//!
//! Ported from `dialogs/dialogs_indexed_list.h/.cpp`.

use std::collections::{BTreeMap, BTreeSet};

use crate::common::SortMode;
use crate::entry::FilterId;
use crate::key::DialogsKey;
use crate::list::{DialogsList, PositionChange};

/// Indexed list over dialogs entries.
///
/// Simplified port: maintains:
/// - `_list` in sorted order,
/// - `_index` buckets by first letter (for fast search filtering).
#[derive(Debug, Clone)]
pub struct IndexedList {
    sort_mode: SortMode,
    filter_id: FilterId,
    list: DialogsList,
    index: BTreeMap<char, DialogsList>,
}

impl IndexedList {
    pub fn new(sort_mode: SortMode, filter_id: FilterId) -> Self {
        Self {
            sort_mode,
            filter_id,
            list: DialogsList::new(sort_mode),
            index: BTreeMap::new(),
        }
    }

    pub fn sort_mode(&self) -> SortMode {
        self.sort_mode
    }

    pub fn set_sort_mode(&mut self, mode: SortMode) {
        self.sort_mode = mode;
        self.list.set_sort_mode(mode);
        for v in self.index.values_mut() {
            v.set_sort_mode(mode);
        }
        // Re-sort all by current mode (simplified): rebuild order by reinserting.
        let keys: Vec<DialogsKey> = self.list.rows().iter().map(|r| r.key.clone()).collect();
        self.list = DialogsList::new(mode);
        self.index.clear();
        for k in keys {
            self.add(k);
        }
    }

    pub fn filter_id(&self) -> FilterId {
        self.filter_id
    }

    pub fn set_filter_id(&mut self, id: FilterId) {
        self.filter_id = id;
    }

    pub fn list(&self) -> &DialogsList {
        &self.list
    }

    pub fn list_mut(&mut self) -> &mut DialogsList {
        &mut self.list
    }

    pub fn empty_list(&self) -> DialogsList {
        DialogsList::new(self.sort_mode)
    }

    /// Add an entry. Updates main list and index buckets.
    pub fn add(&mut self, key: DialogsKey) {
        if !key.entry().has_chat_list_position_for(self.filter_id) {
            // Match tdesktop behavior: only add entries that have a position in this filter/list context.
            // But for generic use, allow adding if caller ensures positions; here we still insert.
        }
        let letters: Vec<char> = key.entry().chat_list_first_letters().into_iter().collect();
        self.list.insert(key.clone());
        for ch in letters {
            self.index
                .entry(ch)
                .or_insert_with(|| DialogsList::new(self.sort_mode))
                .insert(key.clone());
        }
    }

    /// Update after data change (time/pinned etc.).
    pub fn adjust_by_data(&mut self, key: &DialogsKey) -> Option<PositionChange> {
        let change = self.list.adjust_by_data(key)?;
        // Index lists are derived subsets; adjust them too.
        let letters: Vec<char> = key.entry().chat_list_first_letters().into_iter().collect();
        for ch in letters {
            if let Some(list) = self.index.get_mut(&ch) {
                let _ = list.adjust_by_data(key);
            }
        }
        Some(change)
    }

    /// Update after name change.
    pub fn adjust_by_name(&mut self, key: &DialogsKey, old_letters: &BTreeSet<char>) -> Option<PositionChange> {
        let change = self.list.adjust_by_name(key)?;
        let new_letters = key.entry().chat_list_first_letters();
        // Remove from old buckets not present now
        for ch in old_letters {
            if !new_letters.contains(ch) {
                if let Some(list) = self.index.get_mut(ch) {
                    list.remove(key, None);
                    if list.is_empty() {
                        // keep map clean is optional
                    }
                }
            }
        }
        // Add to new buckets
        for ch in &new_letters {
            if !old_letters.contains(ch) {
                self.index
                    .entry(*ch)
                    .or_insert_with(|| DialogsList::new(self.sort_mode))
                    .insert(key.clone());
            } else if let Some(list) = self.index.get_mut(ch) {
                let _ = list.adjust_by_name(key);
            }
        }
        Some(change)
    }

    pub fn remove(&mut self, key: &DialogsKey, replaced_by: Option<&DialogsKey>) -> bool {
        let letters: Vec<char> = key.entry().chat_list_first_letters().into_iter().collect();
        for ch in letters {
            if let Some(list) = self.index.get_mut(&ch) {
                list.remove(key, replaced_by);
            }
        }
        self.list.remove(key, replaced_by)
    }

    /// Peer search in the indexed list (simplified):
    /// returns rows whose entry names contain all query words (case-insensitive).
    pub fn peer_search_results(&self, query: &str) -> Vec<DialogsKey> {
        let words: Vec<String> = query
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .filter(|w| !w.is_empty())
            .collect();
        if words.is_empty() {
            return Vec::new();
        }
        // Start from intersection of first-letter buckets when possible.
        let first_word_letters: Vec<char> = words[0].chars().collect();
        let minimal: Option<Vec<DialogsKey>> = if let Some(&ch) = first_word_letters.first() {
            self.index.get(&ch).map(|l| l.rows().iter().map(|r| r.key.clone()).collect())
        } else {
            None
        };
        let candidates: Vec<DialogsKey> = match minimal {
            Some(v) => v,
            None => self.list.rows().iter().map(|r| r.key.clone()).collect(),
        };
        let mut out = Vec::new();
        for key in candidates {
            let name = key.entry().chat_list_name().to_lowercase();
            let all_found = words.iter().all(|w| name.contains(w));
            if all_found {
                out.push(key);
            }
        }
        out
    }
}
