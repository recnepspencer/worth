use super::*;

mod actions;
mod closeout;
mod layout;
mod materialization;
mod pending;
mod selector_closeout;

pub(crate) fn derive_execution_basis(
    store: StableStoreIdentity,
    selection: &PhysicalSourceSelection,
    freshness: &StoreRecoveryBindingFreshnessSample,
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
    selected_source: &RecoverySelectedSourceInventory,
    successor_candidate: Option<RecoveryObservedSuccessorCandidate>,
    maximum_staging_bytes: u64,
    maximum_dirty_frames: u64,
) -> Result<
    (
        RecoveryStagingLayoutPlan,
        RecoveryPublicationPlan,
        RecoveryQuiescencePlan,
        CandidateMaterializationCost,
        crate::entry::PhysicalRecoveryRootProtocolCounters,
    ),
    ExecutionBasisDenial,
> {
    let pending = pending::admit(
        selection,
        fates,
        redo,
        maximum_staging_bytes,
        maximum_dirty_frames,
    )?;
    let materialization = materialization::collect(&pending);
    let actions = actions::derive(redo, &materialization, pending.staging_generation)?;
    let staging = layout::assemble(
        selection,
        selected_source,
        &pending,
        materialization,
        actions,
    )?;
    closeout::seal(
        store,
        selection,
        freshness,
        fates,
        redo,
        selected_source,
        successor_candidate,
        pending,
        staging,
    )
}

pub(crate) fn requires_successor_candidate(
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
) -> Result<bool, ExecutionBasisDenial> {
    pending::has_pending_projection(fates, redo)
}
