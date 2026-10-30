pub(crate) use {
    worth_signal::facade::branch::AdmittedSignalBranchBasis,
    worth_signal::facade::branch::SignalBranchBasisLifecyclePosture,
};

/// A neutral, test-local view of owner-issued branch truth.
///
/// The production basis remains the only authority. This value deliberately
/// contains only public descriptive fields, so expected comparisons cannot
/// accidentally call back into Signal's basis comparator or private state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NeutralBranchObservation {
    pub(crate) owner_branch_id: u64,
    pub(crate) branch_identity: String,
    pub(crate) graph_instance_id: String,
    pub(crate) definition_basis: u64,
    pub(crate) snapshot_id: Option<u64>,
    pub(crate) restore_snapshot_id: Option<u64>,
    pub(crate) generation: u64,
    pub(crate) lifecycle: SignalBranchBasisLifecyclePosture,
}

pub(crate) fn neutral_basis(basis: &AdmittedSignalBranchBasis) -> NeutralBranchObservation {
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
