//! Observer of actual producer handler entry, independent of demand initiator.
use std::{cell::RefCell, collections::BTreeMap};
use worth_relational::facade::identity::EntityId;
thread_local! { static ENTRIES: RefCell<BTreeMap<(u64, EntityId), u64>> = const { RefCell::new(BTreeMap::new()) }; }
thread_local! {
    static COUPLED_KEYS: RefCell<BTreeMap<(u64, &'static str, EntityId), [u8; 32]>> = const { RefCell::new(BTreeMap::new()) };
}
type LiveScalars = ([u8; 32], Option<[bool; 4]>);
thread_local! {
    static LIVE_COMPLETIONS: RefCell<BTreeMap<(u64, String, EntityId), LiveScalars>> = const { RefCell::new(BTreeMap::new()) };
}
fn observing() -> bool {
    if !crate::domain_computation::execution_observation::is_installed() {
        return false;
    }
    if CLEANUP_REGISTERED.with(|registered| !registered.replace(true)) {
        crate::domain_computation::execution_observation::register_cleanup(clear_observations);
    }
    true
}
/// Passive custody observation of the actual canonical owner output. It changes
/// no key or product decision. One last key per actual coupled route survives
/// abandoned cold candidates, and the existing owned observer clears it.
pub(super) fn record_coupled_key(
    runtime: u64,
    producer: &'static str,
    root: EntityId,
    key: [u8; 32],
) {
    if observing() {
        COUPLED_KEYS.with(|keys| {
            keys.borrow_mut().insert((runtime, producer, root), key);
        });
    }
}
/// D1699: copy scalars only from the actual performing writer's Live path.
/// No receipt, publication, source custody or authority survives this call.
/// Current and prior observations come from the real owner, never an inferred ID.
pub(in crate::domain_computation::primary_graph::application_contribution) fn record_live_completion<
    S: worth_query_installation::facade::ApplicationSchema,
>(
    application: &super::WorthQueryPrimaryGraphApplicationRuntime<S>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    producer: &str,
    root: EntityId,
    receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    before: &crate::basis::WorthQueryProductBranchLease,
    original: Option<
        &crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication,
    >,
) {
    if !observing() {
        return;
    }
    let origins = application
        .on_branch(before.product_branch())
        .select_with_request(execution)
        .ok()
        .and_then(|selected| {
            let publication = receipt.committed_product_publication();
            let described = original?.descriptive();
            Some([
                publication.co_committed_at(selected.product().observation()),
                described.co_committed_at(selected.product().observation()),
                publication.co_committed_at(before.observation()),
                publication == &described,
            ])
        });
    LIVE_COMPLETIONS.with(|records| {
        records.borrow_mut().insert(
            (
                application.runtime.authority_identity().as_u64(),
                producer.to_owned(),
                root,
            ),
            (*receipt.idempotency_binding().key_identity(), origins),
        );
    });
}
pub(super) fn record(runtime: u64, root: EntityId) {
    if !observing() {
        return;
    }
    ENTRIES.with(|entries| {
        let mut entries = entries.borrow_mut();
        *entries.entry((runtime, root)).or_default() += 1;
    });
}
impl<Schema>
    crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    /// Canonical owner-issued coupled keys, for the one closed envelope court.
    #[doc(hidden)]
    pub fn coupled_producer_keys_on_this_thread_for_test(
        &self,
    ) -> Vec<(&'static str, EntityId, [u8; 32], Option<[u8; 32]>)> {
        COUPLED_KEYS.with(|keys| {
            keys.borrow()
                .iter()
                .filter(|((runtime, _, _), _)| {
                    *runtime == self.runtime.authority_identity().as_u64()
                })
                .map(|(address @ (_, producer, root), key)| {
                    let committed = LIVE_COMPLETIONS.with(|records| {
                        records
                            .borrow()
                            .get(&(address.0, address.1.to_owned(), address.2))
                            .map(|(key, _)| *key)
                    });
                    (*producer, *root, *key, committed)
                })
                .collect()
        })
    }
    /// Actual Live origin observations: current, described-current, prior, and
    /// exact descriptive equality with the root source's publication.
    /// Only graph.rs reads this passive scalar view.
    #[doc(hidden)]
    pub fn coupled_producer_origins_on_this_thread_for_test(
        &self,
    ) -> Vec<(String, EntityId, [bool; 4])> {
        LIVE_COMPLETIONS.with(|records| {
            records
                .borrow()
                .iter()
                .filter(|((runtime, _, _), _)| {
                    *runtime == self.runtime.authority_identity().as_u64()
                })
                .filter_map(|((_, producer, root), (_, origins))| {
                    origins.map(|origins| (producer.clone(), *root, origins))
                })
                .collect()
        })
    }
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

// Law 2, scope: passive thread records end with the owned observation lifetime.
thread_local! { static CLEANUP_REGISTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
fn clear_observations() {
    CLEANUP_REGISTERED.with(|registered| registered.set(false));
    ENTRIES.with(|entries| entries.borrow_mut().clear());
    COUPLED_KEYS.with(|keys| keys.borrow_mut().clear());
    LIVE_COMPLETIONS.with(|records| records.borrow_mut().clear());
}
