//! Certification observes published and discarded retention without execution authority.
use super::retained::{RetainedComputation, RetainedPartitions};
use super::WorthQueryPartitionedComputationFullCause as Cause;
use std::sync::Arc;
use worth_execution::{CanonicalBits, ChargedBytes};
use worth_query_declaration::facade::application_operation::application_computation_item_digest;
use worth_query_declaration::facade::application_program::ApplicationComputationPartition;

#[derive(Clone)]
pub struct WorthQueryPublishedComputationStateForTest {
    state: Option<Arc<RetainedComputation>>,
    absence: Option<Cause>,
}

impl std::fmt::Debug for WorthQueryPublishedComputationStateForTest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PublishedComputationState")
            .field("absence", &self.absence)
            .finish_non_exhaustive()
    }
}

thread_local! {
    static DISCARDED: std::cell::RefCell<Vec<Option<Cause>>> = const { std::cell::RefCell::new(Vec::new()) };
    static PUBLISHED: std::cell::RefCell<Vec<WorthQueryPublishedComputationStateForTest>> = const { std::cell::RefCell::new(Vec::new()) };
}

pub(in crate::domain_computation::primary_graph) fn observe_published(
    state: Option<Arc<RetainedComputation>>,
    absence: Option<Cause>,
) {
    // Decisions that ran nothing do not publish computation state.
    if absence != Some(Cause::NotProduced) {
        PUBLISHED.with(|published| {
            published
                .borrow_mut()
                .push(WorthQueryPublishedComputationStateForTest { state, absence })
        });
    }
}

pub fn published_partitioned_computations_on_this_thread_for_test(
) -> Vec<WorthQueryPublishedComputationStateForTest> {
    PUBLISHED.with(|published| std::mem::take(&mut *published.borrow_mut()))
}

/// Retention discarded when a completed ordinary mutation has no producer row.
pub fn discarded_computation_retention_on_this_thread_for_test() -> Vec<Option<Cause>> {
    DISCARDED.with(|discarded| std::mem::take(&mut *discarded.borrow_mut()))
}

pub(in crate::domain_computation) fn observe_discarded(
    computation: &super::SealedComputationRetention,
) {
    let cause = match computation {
        super::SealedComputationRetention::Produced(_) => None,
        super::SealedComputationRetention::Absent(reason) => Some(reason.full_cause()),
    };
    DISCARDED.with(|discarded| discarded.borrow_mut().push(cause));
}

impl WorthQueryPublishedComputationStateForTest {
    /// Compare the fields the next run reads under the supplied item binding
    /// correspondence. Each state's stored digests must encode its actual
    /// items: different installations intentionally encode different authority
    /// identities, even over the same Native entities.
    /// Execution exposes leaves, root and size, but no full tree encoding.
    pub fn same_fields<
        Key: 'static,
        Item: ApplicationComputationPartition + ChargedBytes + 'static,
        Reduced: Clone + ChargedBytes + CanonicalBits + 'static,
    >(
        &self,
        other: &Self,
        same_key: impl Fn(&Key, &Key) -> bool,
        same_item: impl Fn(&Item, &Item) -> bool,
    ) -> bool {
        let (Some(left), Some(right)) = (&self.state, &other.state) else {
            return self.state.is_none() && other.state.is_none() && self.absence == other.absence;
        };
        let left_typed = left
            .typed
            .downcast_ref::<RetainedPartitions<Key, Item, Reduced>>()
            .expect("the certified owner fixes its state types");
        let right_typed = right
            .typed
            .downcast_ref::<RetainedPartitions<Key, Item, Reduced>>()
            .expect("the fresh owner fixes its state types");
        let a = left_typed;
        let b = right_typed;
        let routing = |state: &RetainedPartitions<Key, Item, Reduced>| {
            state
                .routing
                .partitions()
                .map(|partition| {
                    (
                        partition,
                        state.routing.digest(partition).copied(),
                        state.routing.members(partition).collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        };
        let item_keys = |state: &RetainedPartitions<Key, Item, Reduced>| {
            state
                .item_keys
                .iter()
                .map(|(item, call)| (*item, call.declared_units))
                .collect::<Vec<_>>()
        };
        let digests_valid = |state: &RetainedPartitions<Key, Item, Reduced>| {
            state.items.len() == state.digests.len()
                && state.items.iter().all(|(id, item)| {
                    application_computation_item_digest(item, &mut |_| Ok::<(), ()>(()))
                        .ok()
                        .as_ref()
                        == state.digests.get(id)
                })
        };
        digests_valid(a)
            && digests_valid(b)
            && a.items.len() == b.items.len()
            && a.items
                .iter()
                .all(|(id, item)| b.items.get(id).is_some_and(|other| same_item(item, other)))
            && a.membership.declared_units == b.membership.declared_units
            && item_keys(a) == item_keys(b)
            && routing(a) == routing(b)
            && a.partitions.len() == b.partitions.len()
            && a.partitions.iter().all(|(identity, partition)| {
                b.partitions.get(identity).is_some_and(|next| {
                    same_key(&partition.key, &next.key)
                        && partition.key_bytes == next.key_bytes
                        && partition.kernel_units == next.kernel_units
                        && a.tree.leaf(*identity).map(bits) == b.tree.leaf(*identity).map(bits)
                })
            })
            && bits(a.tree.result()) == bits(b.tree.result())
            && a.tree.additional_charged_bytes() == b.tree.additional_charged_bytes()
            && a.charged_bytes() == b.charged_bytes()
            && left.facts.facts().eq(right.facts.facts())
    }
}

fn bits(value: &impl CanonicalBits) -> Vec<u8> {
    let mut bytes = Vec::new();
    assert!(value.visit_canonical_bits(&mut |chunk| {
        bytes.extend_from_slice(chunk);
        true
    }));
    bytes
}
