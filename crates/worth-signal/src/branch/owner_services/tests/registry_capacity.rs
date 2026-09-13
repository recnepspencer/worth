use std::sync::Arc;

use crate::state::SignalBranchId;

use super::super::{
    SignalBranchRegistry, SignalBranchRegistryDenial, SignalOwnerLifecycleState,
    SignalOwnerServiceCounters,
};

#[test]
fn capacity_admission_remains_constant_work_with_many_live_cells() {
    const LIVE_BRANCHES: u64 = 192;
    let counters = Arc::new(SignalOwnerServiceCounters::default());
    let lifecycle = SignalOwnerLifecycleState::new(61, Arc::clone(&counters));
    let admission = lifecycle.admit(61).expect("owner admits capacity work");
    let registry = SignalBranchRegistry::new(&lifecycle, LIVE_BRANCHES as usize + 2, 1);

    for branch in 1..=LIVE_BRANCHES {
        registry
            .reserve(&admission, SignalBranchId(branch))
            .expect("live identity reserves without scanning prior members")
            .install(branch)
            .expect("live state installs");
    }
    let before = counters.snapshot();
    let held = registry
        .reserve(&admission, SignalBranchId(LIVE_BRANCHES + 1))
        .expect("one reservation remains available at scale");
    assert_eq!(registry.live_count(), LIVE_BRANCHES as usize);
    assert_eq!(registry.reservation_count(), 1);
    assert_eq!(
        counters.snapshot().branch_registry_entries_scanned(),
        before.branch_registry_entries_scanned(),
        "capacity admission must use maintained scalar counts"
    );
    assert_eq!(
        registry
            .reserve(&admission, SignalBranchId(LIVE_BRANCHES + 2))
            .unwrap_err(),
        SignalBranchRegistryDenial::ReservationCapacityExhausted {
            maximum_reservations: 1,
        }
    );
    assert_eq!(
        counters.snapshot().branch_registry_entries_scanned(),
        before.branch_registry_entries_scanned(),
        "capacity denial must not inspect existing membership"
    );
    drop(held);
    assert_eq!(registry.reservation_count(), 0);
}
