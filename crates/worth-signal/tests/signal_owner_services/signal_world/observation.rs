use worth_signal::facade::branch::{AdmittedSignalBranchBasis, SignalBranchBasisLifecyclePosture};

/// A neutral, test-local view of owner-issued branch truth.
///
/// The production basis remains the only authority. This value deliberately
/// contains only public descriptive fields, so expected comparisons cannot
/// accidentally call back into Signal's basis comparator or private state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct NeutralBranchObservation {
    pub(super) owner_branch_id: u64,
    pub(super) branch_identity: String,
    pub(super) graph_instance_id: String,
    pub(super) definition_basis: u64,
    pub(super) snapshot_id: Option<u64>,
    pub(super) restore_snapshot_id: Option<u64>,
    pub(super) generation: u64,
    pub(super) lifecycle: SignalBranchBasisLifecyclePosture,
}

pub(super) fn neutral_basis(basis: &AdmittedSignalBranchBasis) -> NeutralBranchObservation {
    let observation = basis.observation();
    let target = observation
        .target()
        .as_basis()
        .expect("owner-issued Signal bases carry a concrete target");
    NeutralBranchObservation {
        owner_branch_id: basis.branch_id().0,
        branch_identity: observation.branch_id().as_str().to_owned(),
        graph_instance_id: target.graph_instance_id().to_owned(),
        definition_basis: target.definition_basis(),
        snapshot_id: target.snapshot_id(),
        restore_snapshot_id: target.restore_snapshot_id(),
        generation: observation.generation().get(),
        lifecycle: basis.descriptor().lifecycle_posture(),
    }
}
