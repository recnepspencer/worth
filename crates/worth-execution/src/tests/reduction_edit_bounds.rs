//! Edit-bound identity refusals precede arithmetic and have no request effects.
use super::*;
use crate::{ReductionEditMemoryBoundError, ReductionPlan, ReductionTree};

#[test]
fn a_delete_bound_names_absence_without_latching_or_charging() {
    let _guard = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
    let lease = authority().request_lease(request(1, 2_000, 100)).unwrap();
    let _held = lease.reserve_retained_memory(17).unwrap();
    let present = PartitionIdentity::new(1);
    let absent = PartitionIdentity::new(2);
    let plan = ReductionPlan::try_from_sorted_unique(vec![present]).unwrap();
    let (tree, _) =
        ReductionTree::try_from_declared(plan, vec![(present, 1_u64)], 0, |left, right| {
            left + right
        })
        .unwrap();
    let before = lease.charged_memory_bytes();
    let outer = crate::backend::run_scope(Some(&lease), 0, 0, |parent| {
        let scope_before = lease.charged_memory_bytes();
        // Even a byte cap that would overflow must not obscure absence.
        assert_eq!(
            tree.checked_delete_memory_bound(absent, u64::MAX),
            Err(ReductionEditMemoryBoundError::UnknownIdentity(absent)),
        );
        assert_eq!(lease.charged_memory_bytes(), scope_before);
        assert_eq!(parent.take_nested_computation_stop(), None);
        assert_eq!(parent.checkpoint(0), Ok(()));
        Ok::<_, KernelFailure<()>>(())
    });
    assert!(outer.result.is_ok());
    assert_eq!(lease.charged_memory_bytes(), before);
}

#[test]
fn edit_bounds_distinguish_identity_refusals_from_arithmetic_overflow() {
    let present = PartitionIdentity::new(1);
    let absent = PartitionIdentity::new(2);
    let plan = ReductionPlan::try_from_sorted_unique(vec![present]).unwrap();
    let (tree, _) =
        ReductionTree::try_from_declared(plan, vec![(present, 1_u64)], 0, |left, right| {
            left + right
        })
        .unwrap();
    assert_eq!(
        tree.checked_insert_memory_bound(present, u64::MAX),
        Err(ReductionEditMemoryBoundError::AlreadyPresent(present))
    );
    assert_eq!(
        tree.checked_update_memory_bound(absent, u64::MAX),
        Err(ReductionEditMemoryBoundError::UnknownIdentity(absent))
    );
    for bound in [
        tree.checked_delete_memory_bound(present, u64::MAX),
        tree.checked_update_memory_bound(present, u64::MAX),
        tree.checked_insert_memory_bound(absent, u64::MAX),
    ] {
        assert_eq!(bound, Err(ReductionEditMemoryBoundError::Overflow));
    }
    assert!(tree.checked_delete_memory_bound(present, 0).is_ok());
    assert!(tree.checked_insert_memory_bound(absent, 0).is_ok());
}
