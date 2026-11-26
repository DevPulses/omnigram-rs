//! Dialog key abstraction.
//!
//! Ported from `dialogs/dialogs_key.h`.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::entry::DialogsEntry;

/// Entry kind carried by a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntryType {
    History,
    Folder,
}

/// Stable key for list/index operations.
///
/// In tdesktop this is `Dialogs::Key` wrapping `not_null<Entry*>`.
/// Here we keep a shared handle to the entry state.
#[derive(Clone, Debug)]
pub struct DialogsKey {
    entry: Arc<DialogsEntry>,
}

impl DialogsKey {
    pub fn new(entry: Arc<DialogsEntry>) -> Self {
        Self { entry }
    }

    pub fn entry(&self) -> &Arc<DialogsEntry> {
        &self.entry
    }

    pub fn entry_type(&self) -> EntryType {
        self.entry.entry_type()
    }

    pub fn is_folder(&self) -> bool {
        matches!(self.entry_type(), EntryType::Folder)
    }
}

impl PartialEq for DialogsKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entry, &other.entry)
    }
}

impl Eq for DialogsKey {}

impl Hash for DialogsKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.entry) as usize).hash(state);
    }
}

impl PartialOrd for DialogsKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DialogsKey {
    fn cmp(&self, other: &Self) -> Ordering {
        (Arc::as_ptr(&self.entry) as usize).cmp(&(Arc::as_ptr(&other.entry) as usize))
    }
}
