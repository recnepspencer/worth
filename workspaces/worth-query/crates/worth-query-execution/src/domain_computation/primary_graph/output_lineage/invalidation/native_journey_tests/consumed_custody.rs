//! Consumed-edge tickets share the invalidation ledger and refund on final drop.
use super::*;
use crate::domain_computation::execution_runtime::source_invalidation::RetainedInvalidationCapacity;
use std::mem::{align_of, size_of};
fn allocated<T>() -> u64 {
    let alignment = align_of::<T>().max(align_of::<usize>());
    let offset = (2 * size_of::<usize>()).div_ceil(alignment) * alignment;
    (offset + size_of::<T>()).div_ceil(alignment) as u64 * alignment as u64
}
#[test]
fn consumed_custody_is_bounded_per_edge_and_returns_on_every_release() {
    let world = installed_authorization_world(true);
    let graph = world.application.runtime.primary_graph().unwrap();
    let handle = graph.integration_handle();
    let owner = &handle.source_owner.invalidation_owner;
    handle.with_runtime_mut(|runtime| {
        let (snapshot_handle, basis) = snapshot(runtime);
        let baseline = owner.resources.retained_capacity_bytes();
        let metadata = 128;
        // Empty fact backing still owns its Arc header and selected native root.
        let fact_alignment =
            align_of::<WorthQueryApplicationObservedFact>().max(align_of::<usize>());
        let facts = (2 * size_of::<usize>()).div_ceil(fact_alignment) * fact_alignment;
        let expected = facts as u64
            + allocated::<PositionedRelationalSnapshot>()
            + basis.branch_id().0.capacity() as u64
            + metadata
            + 2 * allocated::<RetainedInvalidationCapacity>();
        let row = owner
            .retain_consumed_output(&[], &basis, metadata, &mut owner.edit_admission())
            .unwrap();
        assert_eq!(
            owner.resources.retained_capacity_bytes(),
            baseline + expected
        );
        let cancelled_refresh = row.clone();
        assert_eq!(
            owner.resources.retained_capacity_bytes(),
            baseline + expected,
            "a refresh's shared edge reserves once"
        );
        drop(cancelled_refresh);
        assert_eq!(
            owner.resources.retained_capacity_bytes(),
            baseline + expected,
            "cancellation releases its owner without stealing the settled edge"
        );
        let successor = owner
            .retain_consumed_output(&[], &basis, metadata, &mut owner.edit_admission())
            .unwrap();
        assert_eq!(
            owner.resources.retained_capacity_bytes(),
            baseline + 2 * expected
        );
        drop(row);
        assert_eq!(
            owner.resources.retained_capacity_bytes(),
            baseline + expected,
            "supersession releases the displaced edge"
        );
        let demand = successor.clone();
        drop(successor);
        assert_eq!(
            owner.resources.retained_capacity_bytes(),
            baseline + expected,
            "reclamation cannot refund an edge another owner still holds"
        );
        drop(demand);
        assert_eq!(
            owner.resources.retained_capacity_bytes(),
            baseline,
            "the final demand drop refunds the edge"
        );
        // Leave exactly the first ticket's capacity. Refusal of the second
        // ticket must give the already reserved first ticket back.
        let first_ticket = expected - metadata - allocated::<RetainedInvalidationCapacity>();
        let maximum = owner.resources.installation().maximum_retained_bytes;
        let filled = owner
            .resources
            .reserve_retained_capacity(maximum - baseline - first_ticket)
            .unwrap();
        let before = owner.resources.retained_capacity_bytes();
        assert!(owner
            .retain_consumed_output(&[], &basis, metadata, &mut owner.edit_admission())
            .is_err());
        assert_eq!(
            owner.resources.retained_capacity_bytes(),
            before,
            "interrupted edge admission refunds its first reservation"
        );
        drop(filled);
        assert_eq!(owner.resources.retained_capacity_bytes(), baseline);
        runtime
            .snapshots()
            .release_snapshot(&snapshot_handle)
            .unwrap();
    });
}
