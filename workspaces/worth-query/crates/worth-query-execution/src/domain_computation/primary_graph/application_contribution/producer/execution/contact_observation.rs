//! Observer of actual producer handler entry, independent of demand initiator.
use std::{cell::RefCell, collections::BTreeMap};
use worth_relational::facade::identity::EntityId;
thread_local! { static ENTRIES: RefCell<BTreeMap<(u64, EntityId), u64>> = const { RefCell::new(BTreeMap::new()) }; }
pub(super) fn record(runtime: u64, root: EntityId) {
    ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        *entries.entry((runtime, root)).or_default() += 1;
    });
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    #[doc(hidden)]
    pub fn producer_contacts_on_this_thread_for_test(&self) -> u64 {
        ENTRIES.with(|entries| {
            entries
                .borrow()
                .iter()
                .filter(|((runtime, _), _)| *runtime == self.runtime.authority_identity().as_u64())
                .map(|(_, count)| count)
                .sum()
        })
    }
    #[doc(hidden)]
    pub fn producer_contacts_at_root_on_this_thread_for_test(&self, root: EntityId) -> u64 {
        ENTRIES.with(|entries| {
            entries
                .borrow()
                .get(&(self.runtime.authority_identity().as_u64(), root))
                .copied()
                .unwrap_or(0)
        })
    }
}
