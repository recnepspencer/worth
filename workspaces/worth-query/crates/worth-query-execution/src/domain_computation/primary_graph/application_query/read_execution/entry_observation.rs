//! Certification observation of real, synchronous one-shot read kernel entry.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use worth_relational::facade::identity::EntityId;

thread_local! {
    static KERNEL_ENTRIES: Cell<u64> = const { Cell::new(0) };
    static ROOT_ENTRIES: RefCell<BTreeMap<EntityId, u64>> = const { RefCell::new(BTreeMap::new()) };
}

pub(super) fn record_kernel_entry(root: EntityId) {
    ROOT_ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        let count = entries.entry(root).or_default();
        *count = count
            .checked_add(1)
            .expect("root query entry counter overflow");
    });
    KERNEL_ENTRIES.with(|entries| {
        entries.set(
            entries
                .get()
                .checked_add(1)
                .expect("query entry counter overflow"),
        );
    });
}

/// Real kernel entries at an admitted root on this thread.
#[doc(hidden)]
pub fn query_read_kernel_entries_by_root_on_this_thread_for_test() -> BTreeMap<EntityId, u64> {
    ROOT_ENTRIES.with(|entries| entries.borrow().clone())
}

/// Includes attempted reads which later deny. Admission and projection-only
/// activity are outside this measurement; other threads have independent counts.
#[doc(hidden)]
pub fn query_read_kernel_entries_on_this_thread_for_test() -> u64 {
    KERNEL_ENTRIES.with(Cell::get)
}
