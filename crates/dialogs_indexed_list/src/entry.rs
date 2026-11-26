//! Dialog entry state.
//!
//! Ported from `dialogs/dialogs_entry.h/.cpp`.
//! This is a data model of the fields and ordering logic used by dialogs lists.
//! Session/UI notification hooks are represented as plain state transitions.

use std::collections::{BTreeMap, BTreeSet};

use models_peers::PeerId;

use crate::common::{DateTextCache, SortMode, UnreadState};
use crate::key::EntryType;

/// Folder id type used in tdesktop dialogs.
pub type FolderId = u32;

/// Position of an entry in a specific chat list/filter.
#[derive(Debug, Clone, Copy, Default)]
pub struct EntryPosition {
    pub value: f64,
    pub date: i64,
}

impl EntryPosition {
    pub const MIN: f64 = 0.0;
    pub const MAX: f64 = f64::from_bits(0x7FEF_FFFF_FFFF_FFFF); // close to DBL_MAX

    pub fn is_empty(self) -> bool {
        self.value == 0.0
    }
}

/// Internal entry flags (`Dialogs::Entry::Flags` subset used by list logic).
#[derive(Debug, Clone, Copy, Default)]
pub struct EntryFlags(u32);

impl EntryFlags {
    pub const IS_PINNED: u32 = 1 << 0;
    pub const HIDDEN_BY_IS_PINNED: u32 = 1 << 1;

    pub fn contains(self, bit: u32) -> bool {
        (self.0 & bit) != 0
    }

    pub fn set(&mut self, bit: u32, value: bool) {
        if value {
            self.0 |= bit;
        } else {
            self.0 &= !bit;
        }
    }
}

/// Mutable/readable state backing a `DialogsEntry`.
///
/// In tdesktop, `Entry` is a polymorphic base for History/Folder/Sublist.
/// Here we model the shared state required by `DialogsList`/`IndexedList`.
#[derive(Debug, Clone)]
pub struct DialogsEntryState {
    pub entry_type: EntryType,
    pub peer_id: Option<PeerId>,
    pub folder_id: Option<FolderId>,
    /// Primary key for logs/debugging.
    pub key_id: u64,
    /// Name used for indexed search/sort.
    pub name: String,
    /// First letters (lowercase) for indexed search buckets.
    pub first_letters: BTreeSet<char>,
    /// Per-filter chat list positions.
    pub positions: BTreeMap<FilterId, EntryPosition>,
    /// Pinned index in chat list (1-based in tdesktop, 0 = not pinned).
    pub pinned_index: u32,
    /// Is this entry currently in a loaded chat list.
    pub in_chat_list: bool,
    /// Chat list time id (last message/send date id, 0 if not to be shown).
    pub time_id: i32,
    /// Unread state.
    pub unread: UnreadState,
    /// Display unread mark.
    pub marked_unread: bool,
    /// Cached date text.
    pub date_cache: DateTextCache,
    pub flags: EntryFlags,
}

/// Filter id used by tdesktop chat filters. `0` means main list.
pub type FilterId = u32;

impl DialogsEntryState {
    pub fn new_history(key_id: u64, peer_id: PeerId, name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            entry_type: EntryType::History,
            peer_id: Some(peer_id),
            folder_id: None,
            key_id,
            first_letters: compute_first_letters(&name),
            name,
            positions: BTreeMap::new(),
            pinned_index: 0,
            in_chat_list: false,
            time_id: 0,
            unread: UnreadState::default(),
            marked_unread: false,
            date_cache: DateTextCache::default(),
            flags: EntryFlags::default(),
        }
    }

    pub fn new_folder(key_id: u64, folder_id: FolderId, name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            entry_type: EntryType::Folder,
            peer_id: None,
            folder_id: Some(folder_id),
            key_id,
            first_letters: compute_first_letters(&name),
            name,
            positions: BTreeMap::new(),
            pinned_index: 0,
            in_chat_list: false,
            time_id: 0,
            unread: UnreadState::default(),
            marked_unread: false,
            date_cache: DateTextCache::default(),
            flags: EntryFlags::default(),
        }
    }
}

/// Shared dialog entry handle.
#[derive(Debug, Clone)]
pub struct DialogsEntry {
    pub state: std::cell::RefCell<DialogsEntryState>,
}

impl DialogsEntry {
    pub fn new(state: DialogsEntryState) -> Self {
        Self {
            state: std::cell::RefCell::new(state),
        }
    }

    pub fn entry_type(&self) -> EntryType {
        self.state.borrow().entry_type
    }

    pub fn is_folder(&self) -> bool {
        matches!(self.entry_type(), EntryType::Folder)
    }

    pub fn is_pinned_dialog(&self, filter_id: FilterId) -> bool {
        let s = self.state.borrow();
        if filter_id != 0 {
            s.pinned_index > 0
        } else {
            s.flags.contains(EntryFlags::IS_PINNED)
        }
    }

    pub fn pinned_index(&self, filter_id: FilterId) -> u32 {
        let s = self.state.borrow();
        if filter_id == 0 || s.pinned_index > 0 {
            s.pinned_index
        } else {
            0
        }
    }

    pub fn set_pinned_index(&self, filter_id: FilterId, index: u32) {
        let mut s = self.state.borrow_mut();
        if filter_id == 0 {
            s.flags.set(EntryFlags::IS_PINNED, index > 0);
        }
        s.pinned_index = index;
    }

    pub fn cache_pinned_index(&self, _filter_id: FilterId, index: u32) {
        self.state.borrow_mut().pinned_index = index;
    }

    pub fn is_pinned_in_chat_list(&self, filter_id: FilterId) -> bool {
        !self.is_folder() && self.is_pinned_dialog(filter_id)
    }

    pub fn in_chat_list(&self) -> bool {
        self.state.borrow().in_chat_list
    }

    pub fn set_in_chat_list(&self, value: bool) {
        self.state.borrow_mut().in_chat_list = value;
    }

    pub fn chat_list_time_id(&self) -> i32 {
        self.state.borrow().time_id
    }

    pub fn adjusted_chat_list_time_id(&self) -> i32 {
        // Mirrors `Entry::adjustedChatListTimeId()` behavior for server-time ids.
        let time = self.chat_list_time_id();
        if time <= 0 { 0 } else { time }
    }

    pub fn set_chat_list_time_id(&self, value: i32) {
        self.state.borrow_mut().time_id = value;
    }

    pub fn chat_list_name(&self) -> String {
        self.state.borrow().name.clone()
    }

    pub fn set_chat_list_name(&self, name: impl Into<String>) {
        let mut s = self.state.borrow_mut();
        s.name = name.into();
        s.first_letters = compute_first_letters(&s.name);
    }

    pub fn chat_list_first_letters(&self) -> BTreeSet<char> {
        self.state.borrow().first_letters.clone()
    }

    pub fn unread_state(&self) -> UnreadState {
        self.state.borrow().unread
    }

    pub fn marked_unread(&self) -> bool {
        self.state.borrow().marked_unread
    }

    pub fn set_unread_state(&self, unread: UnreadState, marked_unread: bool) {
        let mut s = self.state.borrow_mut();
        s.unread = unread;
        s.marked_unread = marked_unread;
    }

    pub fn sort_key_in_chat_list(&self, mode: SortMode) -> u64 {
        let s = self.state.borrow();
        sort_key_value(mode, s.time_id, s.pinned_index)
    }

    pub fn sort_key_value(&self, mode: SortMode) -> u64 {
        self.sort_key_in_chat_list(mode)
    }

    pub fn pos_in_chat_list(&self, filter_id: FilterId) -> Option<f64> {
        self.state
            .borrow()
            .positions
            .get(&filter_id)
            .map(|p| p.value)
    }

    pub fn date_pos_in_chat_list(&self, filter_id: FilterId) -> Option<i64> {
        self.state
            .borrow()
            .positions
            .get(&filter_id)
            .map(|p| p.date)
    }

    pub fn position(&self, filter_id: FilterId) -> EntryPosition {
        self.state
            .borrow()
            .positions
            .get(&filter_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn set_chat_list_position(&self, filter_id: FilterId, pos: EntryPosition) {
        let mut s = self.state.borrow_mut();
        if pos.is_empty() {
            s.positions.remove(&filter_id);
        } else {
            s.positions.insert(filter_id, pos);
        }
    }

    pub fn has_chat_list_position(&self) -> bool {
        !self.state.borrow().positions.is_empty()
    }

    pub fn has_chat_list_position_for(&self, filter_id: FilterId) -> bool {
        self.pos_in_chat_list(filter_id).is_some()
    }

    pub fn update_chat_list_sort_position(&self) {
        // In tdesktop this updates the `PositionChange` in Data::Session changes.
        // In this crate, callers observe list/index return values.
    }
}

/// Compute first letters for indexed search.
///
/// Mirrors the simplified logic in tdesktop: lowercase first char of words.
/// This is a pragmatic approximation; real tdesktop uses text utilities and locale rules.
pub fn compute_first_letters(name: &str) -> BTreeSet<char> {
    let mut out = BTreeSet::new();
    for word in name.split(|c: char| !c.is_alphanumeric()) {
        if let Some(ch) = word.chars().next() {
            for lc in ch.to_lowercase() {
                if lc.is_alphanumeric() {
                    out.insert(lc);
                }
            }
        }
    }
    out
}

/// Sort key as used by `DialogsList`.
///
/// Pinned entries get the highest band. Folders are kept above regular entries
/// in date mode in tdesktop through `Folder` fixed position; for generic entries
/// we use pinned/time ordering.
pub fn sort_key_value(mode: SortMode, time_id: i32, pinned_index: u32) -> u64 {
    match mode {
        SortMode::Name => 0,
        SortMode::Date | SortMode::Add => {
            if pinned_index > 0 {
                // Preserve pinned order by inverting index (smaller index = higher key).
                (0xFFFF_FFFF_u64 << 32) | (u32::MAX as u64 - pinned_index as u64)
            } else {
                (time_id.max(0) as u64) << 16
            }
        }
    }
}
