//! Both retention gates preserve the predecessor's exact lane obligations.
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    branch::AdmittedRelationalBranchBasis,
    change_source::RelationalRetainedObservationRetirementProgress as Progress,
    durability::RelationalRecoveredPublicationRetirement,
};

pub(super) fn assert_busy_and_first_release(
    task: &mut RelationalRecoveredPublicationRetirement,
    basis: &AdmittedRelationalBranchBasis,
    request: &ExecutionRequest<'_, '_>,
) {
    let ([cut, lane], progress, releases) = task
        .advance_with_retention_gate_held_for_test(basis, request)
        .unwrap();
    assert_eq!(cut.progress, Progress::Busy);
    assert_eq!(cut.after.counters, cut.before.counters);
    assert_eq!(lane.progress, Progress::Busy);
    assert!(lane.before.root_obligations.is_some());
    assert_eq!(lane.after.lane_obligations, lane.before.lane_obligations);
    assert_eq!(lane.after.root_obligations, lane.before.root_obligations);
    assert_eq!(progress, Progress::Advanced);
    assert_eq!(releases, cut.before.counters.external_pin_releases + 1);
}
