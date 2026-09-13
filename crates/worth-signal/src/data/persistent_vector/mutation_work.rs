//! Structural work of writes to existing indices; payload cloning is separate.
use super::{PersistentVector, PersistentVectorStorage};

impl<T: Clone, const PAGE_LEN: usize> PersistentVector<T, PAGE_LEN> {
    /// Covers repeated get_mut calls without an intervening append or fork.
    /// Use the maximum possible changed-page population, so earlier writes in
    /// the same batch cannot invalidate this bound. No payload is inspected.
    pub(crate) fn indexed_mutation_work_bound(&self, writes: usize) -> Option<usize> {
        if writes == 0 {
            return Some(0);
        }
        let per_write = match &self.storage {
            PersistentVectorStorage::Exclusive(_) => Some(1),
            PersistentVectorStorage::ForkShared { len, .. } => {
                // contains_key, insertion/splitting, get_mut and path COW in
                // the installed im tree. ordered_lookup_steps includes the
                // 64-entry node width; 32 allows movement and Arc bookkeeping.
                let tree =
                    crate::data::retained_storage::ordered_lookup_steps(len.div_ceil(PAGE_LEN));
                // ForkPage clone only clones Arc entries. Binary search,
                // override insertion/shifting and page-vector growth touch at
                // most PAGE_LEN entries, independently of T's heap payload.
                tree.checked_mul(32)
                    .and_then(|n| PAGE_LEN.checked_mul(8)?.checked_add(32)?.checked_add(n))
            }
        };
        per_write?.checked_mul(writes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_bound_covers_page_growth_without_inspecting_payloads() {
        #[derive(Debug)]
        struct Payload;
        impl Clone for Payload {
            fn clone(&self) -> Self {
                panic!("structural admission must not clone payloads")
            }
        }
        let mut values: PersistentVector<Payload, 4> = (0..17).map(|_| Payload).collect();
        assert_eq!(values.indexed_mutation_work_bound(7), Some(7));
        let retained = values.fork_persistent();
        let bound = values.indexed_mutation_work_bound(7).unwrap();
        assert!(bound > 7);
        for i in 0..17 {
            values.replace_discard(i, Payload);
            assert_eq!(values.indexed_mutation_work_bound(7), Some(bound));
        }
        assert_eq!(retained.len(), 17);
        assert_eq!(values.indexed_mutation_work_bound(0), Some(0));
        assert_eq!(values.indexed_mutation_work_bound(usize::MAX), None);
    }
}
