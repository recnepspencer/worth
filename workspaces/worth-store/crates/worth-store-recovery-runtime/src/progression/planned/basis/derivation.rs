use super::*;

mod actions;
pub(super) mod closeout;
mod layout;
mod materialization;
mod pending;
mod release_head;
mod selector_closeout;

pub(crate) fn derive_execution_basis(
    store: StableStoreIdentity,
    selection: &PhysicalSourceSelection,
    freshness: &StoreRecoveryBindingFreshnessSample,
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
    historical_consumed: &HistoricalConsumedOperationSet,
    selected_source: &RecoverySelectedSourceInventory,
    verified_drops: &[worth_store_physical_format::PersistedRecordIdentity],
    validated_manifest_cleanup: Option<crate::orchestration::ValidatedManifestResidueCleanup>,
    release_head_replay: Option<
        &worth_store_recovery_physics::VerifiedSelectedReleaseHeadReplayV14,
    >,
    successor_candidate: Option<RecoveryObservedSuccessorCandidate>,
    maximum_manifest_entries: u64,
    maximum_staging_bytes: u64,
    maximum_dirty_frames: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<
    (
        RecoveryStagingLayoutPlan,
        RecoveryPublicationPlan,
        RecoveryQuiescencePlan,
        CandidateMaterializationCost,
        crate::entry::PhysicalRecoveryRootProtocolCounters,
        u64,
    ),
    ExecutionBasisDenial,
> {
    let pending = pending::admit(
        selection,
        fates,
        redo,
        historical_consumed,
        maximum_staging_bytes,
        maximum_dirty_frames,
        allowance,
    )?;
    if validated_manifest_cleanup.is_some()
        && (!pending.projections.is_empty()
            || !pending.source_copies.is_empty()
            || !verified_drops.is_empty())
    {
        return Err(ExecutionBasisDenial::Invalid);
    }
    let materialization = materialization::collect(&pending, allowance)?;
    let actions = actions::derive(
        redo,
        &materialization,
        pending.staging_generation,
        allowance,
    )?;
    let staging = layout::assemble(
        selection,
        selected_source,
        verified_drops,
        validated_manifest_cleanup,
        release_head_replay,
        &pending,
        materialization,
        actions,
        allowance,
    )?;
    let (staging, publication, quiescence, candidate_cost, counters) = closeout::seal(
        store,
        selection,
        freshness,
        fates,
        redo,
        historical_consumed,
        selected_source,
        verified_drops,
        validated_manifest_cleanup,
        successor_candidate,
        pending,
        staging,
        maximum_manifest_entries,
        maximum_staging_bytes,
        allowance,
    )?;
    let planning_peak = allowance.peak();
    Ok((
        staging,
        publication,
        quiescence,
        candidate_cost,
        counters,
        planning_peak,
    ))
}

pub(crate) fn requires_successor_candidate(
    selection: &PhysicalSourceSelection,
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
    historical_consumed: &HistoricalConsumedOperationSet,
) -> Result<bool, ExecutionBasisDenial> {
    pending::has_pending_projection(selection, fates, redo, historical_consumed)
}
