use super::super::staging_cost::preflight_staging_cost;
use super::super::*;

pub(super) struct PendingProjectionBasis<'plan> {
    pub(super) checkpoint: PhysicalCheckpointIdentity,
    pub(super) source_generation: u64,
    pub(super) staging_generation: u64,
    pub(super) projections: Vec<&'plan worth_store_recovery_physics::PhysicalRedoProjection>,
    pub(super) source_copies: Vec<&'plan worth_store_recovery_physics::PhysicalExtentCopyAdmission>,
    pub(super) allocated_bytes: u64,
}

pub(super) fn admit<'plan>(
    selection: &PhysicalSourceSelection,
    fates: &ReconciledOperationFates,
    redo: &'plan ImmutablePhysicalRedoPlan,
    historical_consumed: &HistoricalConsumedOperationSet,
    maximum_staging_bytes: u64,
    maximum_dirty_frames: u64,
    allowance: &mut PlanningResidentAllowance,
) -> Result<PendingProjectionBasis<'plan>, ExecutionBasisDenial> {
    let checkpoint = selection
        .checkpoint()
        .ok_or(ExecutionBasisDenial::Invalid)?
        .checkpoint()
        .source()
        .identity();
    let source_generation = selection.root().selected().selector().root_generation();
    let staging_generation = source_generation
        .checked_add(1)
        .ok_or(ExecutionBasisDenial::Invalid)?;
    let projections = pending_projections(fates, redo, historical_consumed, allowance)?;
    if projections.iter().any(|projection| {
        projection.materialization().source_root_generation() != source_generation
    }) {
        return Err(ExecutionBasisDenial::Invalid);
    }
    let mut allocated_bytes = preflight_staging_cost(
        &projections,
        maximum_staging_bytes,
        maximum_dirty_frames,
        allowance,
    )?;
    let source_copies = pending_source_copies(selection, redo, allowance)?;
    for copy in &source_copies {
        let intent = copy.recipe().intent();
        let frames = u64::from(intent.chunk_count()) + 1;
        let charge = copy
            .projection()
            .root_state()
            .root_publication_allocation_bytes()
            .checked_add(u64::from(intent.maximum_frame_bytes()) * 4)
            .and_then(|bytes| {
                bytes.checked_add(
                    frames
                        * (std::mem::size_of::<crate::entry::PhysicalRecoveryStagingSettlement>()
                            as u64
                            + 256),
                )
            })
            .ok_or(ExecutionBasisDenial::Invalid)?;
        allocated_bytes = allocated_bytes
            .checked_add(charge)
            .ok_or(ExecutionBasisDenial::Invalid)?;
        if allocated_bytes > maximum_staging_bytes {
            return Err(ExecutionBasisDenial::StagingBytes {
                observed: allocated_bytes,
            });
        }
    }
    Ok(PendingProjectionBasis {
        checkpoint,
        source_generation,
        staging_generation,
        projections,
        source_copies,
        allocated_bytes,
    })
}

fn pending_projections<'plan>(
    fates: &ReconciledOperationFates,
    redo: &'plan ImmutablePhysicalRedoPlan,
    historical_consumed: &HistoricalConsumedOperationSet,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<&'plan worth_store_recovery_physics::PhysicalRedoProjection>, ExecutionBasisDenial>
{
    let mut count = 0_usize;
    for projection in redo.projections() {
        count += usize::from(projection_pending(projection, fates, historical_consumed)?);
    }
    let mut pending = allowance.reserve(count)?;
    for projection in redo.projections() {
        if projection_pending(projection, fates, historical_consumed)? {
            pending.push(projection);
        }
    }
    Ok(pending)
}

pub(super) fn has_pending_projection(
    selection: &PhysicalSourceSelection,
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
    historical_consumed: &HistoricalConsumedOperationSet,
) -> Result<bool, ExecutionBasisDenial> {
    let mut required = false;
    for projection in redo.projections() {
        required |= projection_pending(projection, fates, historical_consumed)?;
    }
    for copy in redo.source_copies() {
        required |= source_copy_pending(selection, copy)?;
    }
    Ok(required)
}

fn projection_pending(
    projection: &worth_store_recovery_physics::PhysicalRedoProjection,
    fates: &ReconciledOperationFates,
    historical_consumed: &HistoricalConsumedOperationSet,
) -> Result<bool, ExecutionBasisDenial> {
    let fate = fates
        .operations()
        .iter()
        .find(|fate| fate.identity().idempotency() == projection.operation())
        .ok_or(ExecutionBasisDenial::Invalid)?
        .fate();
    Ok(
        fate == worth_store_recovery_physics::RecoveryOperationFate::Indeterminate
            && !historical_consumed.contains(projection.operation()),
    )
}

fn pending_source_copies<'a>(
    selection: &PhysicalSourceSelection,
    redo: &'a ImmutablePhysicalRedoPlan,
    allowance: &mut PlanningResidentAllowance,
) -> Result<Vec<&'a worth_store_recovery_physics::PhysicalExtentCopyAdmission>, ExecutionBasisDenial>
{
    let mut count = 0_usize;
    for copy in redo.source_copies() {
        count += usize::from(source_copy_pending(selection, copy)?);
    }
    let mut pending = allowance.reserve(count)?;
    for copy in redo.source_copies() {
        if source_copy_pending(selection, copy)? {
            pending.push(copy);
        }
    }
    Ok(pending)
}

fn source_copy_pending(
    selection: &PhysicalSourceSelection,
    copy: &worth_store_recovery_physics::PhysicalExtentCopyAdmission,
) -> Result<bool, ExecutionBasisDenial> {
    if copy.fate() != worth_store_recovery_physics::RecoveryOperationFate::Indeterminate {
        return Ok(false);
    }
    let generation = selection.root().selected().selector().root_generation();
    let source = copy.projection().source_root_generation();
    let intent = copy.recipe().intent();
    if source
        .checked_add(1)
        .is_some_and(|result| generation > result)
    {
        // Completion verified either the exact durable Published resolution
        // or the canonical historical root and retained copy bytes. Neither
        // historical posture needs another publication replay.
        return Ok(false);
    }
    let expected = if generation == source {
        intent.source()
    } else if source.checked_add(1) == Some(generation) {
        intent.destination()
    } else {
        return Err(ExecutionBasisDenial::Invalid);
    };
    if !selection
        .page_facts()
        .placements()
        .contains(&CurrentPhysicalRecordPlacement::Extent(expected))
    {
        return Err(ExecutionBasisDenial::Invalid);
    }
    Ok(generation == source)
}
