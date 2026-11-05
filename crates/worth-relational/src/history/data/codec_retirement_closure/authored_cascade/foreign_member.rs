//! Extra roster entries cannot expand closure; classify their actual provenance.
use super::*;

pub(super) fn classify(
    execution: &ExecutionRequest<'_, '_>,
    runtime: &RelationalRuntime,
    basis: &AdmittedRelationalBranchBasis,
    id: EntityId,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<Cause, TransactionCommitError> {
    // A foreign owning publication is a current fact. Read only this member's
    // claims; never follow its scope or descendants into the selected closure.
    let roster = crate::publication::ownership::read_selected(
        execution,
        basis,
        &[RecordRef::Entity(id)],
        &[],
        None,
        context,
    )?;
    for origin in roster.origins() {
        charge(context, origin.members.len().saturating_add(1)).map_err(stopped)?;
        if origin
            .members
            .iter()
            .any(|m| m.id == id && m.posture != Posture::Retire)
        {
            return Ok(Cause::DifferentSourcePublication);
        }
    }
    // Independent producers deliberately have no ownership roster. Their birth
    // provenance is historical evidence, not an alternative current owner set.
    let Some(partition) = basis
        .inner
        .root
        .partition_state(id.partition_id)
        .map_err(|cause| stopped(Stop::from(cause)))?
    else {
        return Ok(Cause::ExtraEntity);
    };
    let Some(created) = partition.entity_arena.created_at_for_slot(id.slot_index()) else {
        return Ok(Cause::ExtraEntity);
    };
    let Some(birth) = runtime.history.canonical_envelope_for_version(created) else {
        return Ok(Cause::ExtraEntity);
    };
    for origin in birth.output_origins() {
        charge(context, origin.members.len().saturating_add(1)).map_err(stopped)?;
        if !origin.source_scope_owned && origin.members.iter().any(|m| m.id == id) {
            return Ok(Cause::UnrelatedPublication);
        }
    }
    Ok(Cause::ExtraEntity)
}
