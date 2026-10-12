//! Certification observation of real, synchronous one-shot read kernel entry.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use worth_relational::facade::identity::EntityId;

thread_local! {
    static KERNEL_ENTRIES: Cell<u64> = const { Cell::new(0) };
    static ROOT_ENTRIES: RefCell<BTreeMap<EntityId, u64>> = const { RefCell::new(BTreeMap::new()) };
}

pub(super) fn record_kernel_entry(root: EntityId) {
    if !crate::domain_computation::execution_observation::is_installed() {
        return;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
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

#[cfg(test)]
mod observation_tests {
    use super::*;
    use worth_relational::facade::identity::PartitionId;
    #[test]
    fn an_unobserved_kernel_entry_creates_no_root_record() {
        let root = EntityId::new(PartitionId::main(), 29, 1);
        let before = query_read_kernel_entries_on_this_thread_for_test();
        record_kernel_entry(root);
        assert_eq!(query_read_kernel_entries_on_this_thread_for_test(), before);
        assert!(!query_read_kernel_entries_by_root_on_this_thread_for_test().contains_key(&root));
        let outer =
            crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        let inner =
            crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        record_kernel_entry(root);
        assert_eq!(
            query_read_kernel_entries_on_this_thread_for_test(),
            before + 1
        );
        drop(outer);
        record_kernel_entry(root);
        assert_eq!(
            query_read_kernel_entries_on_this_thread_for_test(),
            before + 2
        );
        drop(inner);
        record_kernel_entry(root);
        assert_eq!(query_read_kernel_entries_on_this_thread_for_test(), 0);
        assert!(query_read_kernel_entries_by_root_on_this_thread_for_test().is_empty());
    }
}

// Law 2, scope: passive thread records end with the owned observation lifetime.
thread_local! { static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    KERNEL_ENTRIES.with(|entries| entries.set(0));
    ROOT_ENTRIES.with(|entries| entries.borrow_mut().clear());
}
