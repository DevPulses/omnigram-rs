//! Dialogs indexed list and entry abstractions.
//!
//! Ported from Telegram Desktop:
//! - `Telegram/SourceFiles/dialogs/dialogs_entry.h/.cpp`
//! - `Telegram/SourceFiles/dialogs/dialogs_indexed_list.h/.cpp`
//! - `Telegram/SourceFiles/dialogs/dialogs_key.h`
//! - `Telegram/SourceFiles/dialogs/dialogs_list.h/.cpp`
//!
//! This crate models the core sorting/indexing/search behavior of dialog lists.
//! UI-specific painting, Qt signals, and Data::Session dependencies are
//! intentionally omitted; data-facing hooks are represented as plain Rust traits
//! and structs.

pub mod common;
pub mod entry;
pub mod indexed_list;
pub mod key;
pub mod list;

pub use common::{DateTextCache, SortMode, UnreadState};
pub use entry::{DialogsEntry, DialogsEntryState, EntryPosition, FolderId};
pub use indexed_list::IndexedList;
pub use key::{DialogsKey, EntryType};
pub use list::{DialogsList, PositionChange, SortKey};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{DialogsEntryState, FilterId};
    use models_peers::{PeerId, UserId};
    use std::collections::BTreeSet;
    use std::sync::Arc;

    fn make_key(id: u64, name: &str, date: i32) -> DialogsKey {
        let peer = PeerId::from_user(UserId::new(id));
        let state = DialogsEntryState::new_history(id, peer, name);
        let entry = Arc::new(DialogsEntry::new(state));
        entry.set_chat_list_time_id(date);
        DialogsKey::new(entry)
    }

    #[test]
    fn list_orders_by_date_desc() {
        let mut list = DialogsList::new(SortMode::Date);
        let a = make_key(1, "Alpha", 100);
        let b = make_key(2, "Beta", 200);
        let c = make_key(3, "Gamma", 150);
        list.insert(a.clone());
        list.insert(b.clone());
        list.insert(c.clone());
        let ids: Vec<u64> = list
            .rows()
            .iter()
            .map(|r| r.key.entry().state.borrow().key_id)
            .collect();
        assert_eq!(ids, vec![2, 3, 1]);
    }

    #[test]
    fn list_pinned_stays_above_non_pinned() {
        let mut list = DialogsList::new(SortMode::Date);
        let a = make_key(1, "Alpha", 100);
        let b = make_key(2, "Beta", 200);
        let c = make_key(3, "Gamma", 300);

        a.entry().set_chat_list_time_id(100);
        a.entry().set_pinned_index(0, 2);
        b.entry().set_chat_list_time_id(200);
        b.entry().set_pinned_index(0, 1);

        list.insert(b.clone());
        list.insert(c.clone());
        list.insert(a.clone());
        let ids: Vec<u64> = list
            .rows()
            .iter()
            .map(|r| r.key.entry().state.borrow().key_id)
            .collect();
        assert_eq!(ids, vec![2, 1, 3]);
    }

    #[test]
    fn list_adjust_by_data_reorders() {
        let mut list = DialogsList::new(SortMode::Date);
        let a = make_key(1, "Alpha", 100);
        let b = make_key(2, "Beta", 200);
        list.insert(a.clone());
        list.insert(b.clone());
        assert_eq!(list.rows()[0].key.entry().state.borrow().key_id, 2);

        a.entry().set_chat_list_time_id(300);
        let changed = list.adjust_by_data(&a);
        assert!(changed.is_some());
        assert!(changed.unwrap().moved);
        assert_eq!(list.rows()[0].key.entry().state.borrow().key_id, 1);
    }

    #[test]
    fn list_remove_and_find() {
        let mut list = DialogsList::new(SortMode::Date);
        let a = make_key(1, "Alpha", 100);
        list.insert(a.clone());
        assert!(list.find_row(&a).is_some());
        assert_eq!(list.len(), 1);
        assert!(list.remove(&a, None));
        assert!(list.find_row(&a).is_none());
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn indexed_list_peer_search_finds_by_prefix() {
        let mut idx = IndexedList::new(SortMode::Date, 0 as FilterId);
        let a = make_key(1, "Alpha", 100);
        let b = make_key(2, "Beta", 200);
        let g = make_key(3, "Gamma", 150);
        idx.add(a.clone());
        idx.add(b.clone());
        idx.add(g.clone());

        let res = idx.peer_search_results("alp");
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].entry().state.borrow().key_id, 1);

        let res2 = idx.peer_search_results("be ga");
        assert!(res2.is_empty());
    }

    #[test]
    fn indexed_list_adjust_by_data_keeps_order() {
        let mut idx = IndexedList::new(SortMode::Date, 0 as FilterId);
        let a = make_key(1, "Alpha", 100);
        let b = make_key(2, "Beta", 200);
        idx.add(a.clone());
        idx.add(b.clone());
        assert_eq!(idx.list().rows()[0].key.entry().state.borrow().key_id, 2);

        a.entry().set_chat_list_time_id(300);
        let change = idx.adjust_by_data(&a);
        assert!(change.is_some());
        assert!(change.unwrap().moved);
        assert_eq!(idx.list().rows()[0].key.entry().state.borrow().key_id, 1);
    }

    #[test]
    fn indexed_list_adjust_by_name_updates_buckets() {
        let mut idx = IndexedList::new(SortMode::Date, 0 as FilterId);
        let a = make_key(1, "Alpha", 100);
        idx.add(a.clone());
        let old_letters: BTreeSet<char> = a.entry().chat_list_first_letters();
        a.entry().set_chat_list_name("Zeta");
        let _ = idx.adjust_by_name(&a, &old_letters);
        let res_a = idx.peer_search_results("alp");
        assert!(res_a.is_empty());
        let res_z = idx.peer_search_results("zet");
        assert_eq!(res_z.len(), 1);
    }

    #[test]
    fn sort_key_pinned_above_non_pinned() {
        let e1 = Arc::new(DialogsEntry::new(DialogsEntryState::new_history(
            1,
            PeerId::from_user(UserId::new(1)),
            "A",
        )));
        e1.set_chat_list_time_id(100);
        e1.set_pinned_index(0, 1);
        let e2 = Arc::new(DialogsEntry::new(DialogsEntryState::new_history(
            2,
            PeerId::from_user(UserId::new(2)),
            "B",
        )));
        e2.set_chat_list_time_id(999);

        let k1 = e1.sort_key_in_chat_list(SortMode::Date);
        let k2 = e2.sort_key_in_chat_list(SortMode::Date);
        assert!(k1 > k2, "pinned should sort above non-pinned in date mode");
    }
}
