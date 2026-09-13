use super::super::staging_cost::preflight_staging_cost;
use super::super::*;

pub(super) struct PendingProjectionBasis<'plan> {
    pub(super) checkpoint: PhysicalCheckpointIdentity,
    pub(super) source_generation: u64,
    pub(super) staging_generation: u64,
    pub(super) projections: Vec<&'plan worth_store_recovery_physics::PhysicalRedoProjection>,
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
    let allocated_bytes =
        preflight_staging_cost(&projections, maximum_staging_bytes, maximum_dirty_frames)?;
    Ok(PendingProjectionBasis {
        checkpoint,
        source_generation,
        staging_generation,
        projections,
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
    fates: &ReconciledOperationFates,
    redo: &ImmutablePhysicalRedoPlan,
) -> Result<bool, ExecutionBasisDenial> {
    pending_projections(fates, redo).map(|pending| !pending.is_empty())
}
