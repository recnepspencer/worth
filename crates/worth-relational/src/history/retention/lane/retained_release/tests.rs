//! Actual gate contention assertions, borrowing the registered caller's fixture.
use super::*;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::change_source::retained_observation::retirement::RelationalRetainedObservationRetirement;
use crate::change_source::retained_observation::retirement::RelationalRetainedObservationRetirementProgress as Progress;

pub(crate) fn assert_external_cut_and_lane_busy(
    task: &mut RelationalRetainedObservationRetirement,
    basis: &AdmittedRelationalBranchBasis,
    request: &ExecutionRequest<'_, '_>,
) {
    let before = basis
        .inner
        .retention_binding
        .counters()
        .observation_releases;
    assert_recovered_cut_and_lane_busy(basis, || task.advance(request));
    assert_eq!(
        basis
            .inner
            .retention_binding
            .counters()
            .observation_releases,
        before
    );
}

/// The same original release must survive contention at either native gate.
pub(crate) fn assert_recovered_cut_and_lane_busy(
    basis: &AdmittedRelationalBranchBasis,
    mut advance: impl FnMut() -> Progress,
) {
    let binding = &basis.inner.retention_binding;
    let owner = binding.owner.upgrade().unwrap();
    let before = binding.counters();
    let cut = owner.external_cut.lock().unwrap();
    assert_eq!(advance(), Progress::Busy);
    assert_eq!(binding.counters(), before);
    drop(cut);
    let lane = binding.lane.state.lock().unwrap();
    let obligations = lane.obligations.len();
    let roots = *lane.root_obligations.get(&basis.inner.root.id()).unwrap();
    assert_eq!(advance(), Progress::Busy);
    assert_eq!(lane.obligations.len(), obligations);
    assert_eq!(
        *lane.root_obligations.get(&basis.inner.root.id()).unwrap(),
        roots
    );
    drop(lane);
    assert_eq!(advance(), Progress::Advanced);
    assert_eq!(
        binding.counters().external_pin_releases,
        before.external_pin_releases + 1
    );
}
