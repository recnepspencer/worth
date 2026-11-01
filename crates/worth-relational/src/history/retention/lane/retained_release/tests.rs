//! Actual gate contention assertions, borrowing the registered caller's fixture.
use super::*;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::change_source::{
    RelationalRetainedObservationRetirement,
    RelationalRetainedObservationRetirementProgress as Progress,
};

pub(crate) fn assert_external_cut_and_lane_busy(
    task: &mut RelationalRetainedObservationRetirement,
    basis: &AdmittedRelationalBranchBasis,
    request: &ExecutionRequest<'_, '_>,
) {
    let binding = &basis.inner.retention_binding;
    let owner = binding.owner.upgrade().unwrap();
    let before = binding.counters();
    let cut = owner.external_cut.lock().unwrap();
    assert_eq!(task.advance(request), Progress::Busy);
    assert_eq!(binding.counters(), before);
    drop(cut);
    let lane = binding.lane.state.lock().unwrap();
    let obligations = lane.obligations.len();
    let root_count = *lane.root_obligations.get(&basis.inner.root.id()).unwrap();
    assert_eq!(task.advance(request), Progress::Busy);
    assert_eq!(lane.obligations.len(), obligations);
    assert_eq!(
        *lane.root_obligations.get(&basis.inner.root.id()).unwrap(),
        root_count
    );
    drop(lane);
    assert_eq!(task.advance(request), Progress::Advanced);
    let after = binding.counters();
    assert_eq!(
        after.external_pin_releases,
        before.external_pin_releases + 1
    );
    assert_eq!(after.observation_releases, before.observation_releases);
}
