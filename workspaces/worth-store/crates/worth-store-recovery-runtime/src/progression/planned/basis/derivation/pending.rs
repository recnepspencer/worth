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
    maximum_staging_bytes: u64,
    maximum_dirty_frames: u64,
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
    let projections = pending_projections(fates, redo)?;
    if projections.iter().any(|projection| {
        projection.materialization().source_root_generation() != source_generation
    }) {
        return Err(ExecutionBasisDenial::Invalid);
    }
    let mut allocated_bytes =
        preflight_staging_cost(&projections, maximum_staging_bytes, maximum_dirty_frames)?;
    let source_copies = pending_source_copies(selection, redo)?;
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
) -> Result<Vec<&'plan worth_store_recovery_physics::PhysicalRedoProjection>, ExecutionBasisDenial>
{
    let mut pending = Vec::new();
    for projection in redo.projections() {
        let fate = fates
            .operations()
            .iter()
            .find(|fate| fate.identity().idempotency() == projection.operation())
            .ok_or(ExecutionBasisDenial::Invalid)?
            .fate();
        if fate == worth_store_recovery_physics::RecoveryOperationFate::Indeterminate {
            pending.push(projection);
        }
    }
    Ok(pending)
}

pub(super) fn has_pending_projection(
    selection: &PhysicalSourceSelection,
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
) -> Result<bool, ExecutionBasisDenial> {
    Ok(!pending_projections(fates, redo)?.is_empty()
        || !pending_source_copies(selection, redo)?.is_empty())
}

fn pending_source_copies<'a>(
    selection: &PhysicalSourceSelection,
    redo: &'a ImmutablePhysicalRedoPlan,
) -> Result<Vec<&'a worth_store_recovery_physics::PhysicalExtentCopyAdmission>, ExecutionBasisDenial>
{
    let generation = selection.root().selected().selector().root_generation();
    let mut pending = Vec::new();
    for copy in redo.source_copies().iter().filter(|copy| {
        copy.fate() == worth_store_recovery_physics::RecoveryOperationFate::Indeterminate
    }) {
        let source = copy.projection().source_root_generation();
        let intent = copy.recipe().intent();
        if source
            .checked_add(1)
            .is_some_and(|result| generation > result)
        {
            // Completion verified an exact durable Published resolution before
            // deriving this execution basis. Historical publication needs no replay.
            continue;
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
        if generation == source {
            pending.push(copy);
        }
    }
    Ok(pending)
}
