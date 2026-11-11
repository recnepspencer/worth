//! Entity slot transitions from actual retained Native head custody.
use super::*;
pub(super) fn select(
    workspace: &mut MutationWorkspace<'_>,
    source: &crate::branch::RelationalBranchRoot,
    id: EntityId,
    version: VersionId,
    outcome: &mut MutationOutcome,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), MutationApplyDenial> {
    let record = RecordRef::Entity(id);
    let arena = source
        .get_current_partition(id.partition_id)
        .ok_or_else(|| stale(record.clone(), "selected entity partition unavailable"))?
        .entity_arena();
    let selected = arena
        .get(&id)
        .filter(|slot| slot.is_live())
        .ok_or_else(|| stale(record.clone(), "selected full-GEN entity unavailable"))?;
    let kind = selected
        .kind_id()
        .ok_or_else(|| stale(record.clone(), "selected entity kind unavailable"))?;
    let payload = selected.extra().authoritative_aspect_state();
    crate::execution::request_stage::checkpoint(
        context,
        payload
            .as_ref()
            .map_or(0, |p| p.owned_allocation_capacity_bytes()) as u64,
    )
    .map_err(request)?;
    let before = workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut_for_request(id.partition_id, &context.execution)
            .map_err(request)?;
        let before = partition
            .entity_arena
            .get(&id)
            .filter(|slot| slot.is_live())
            .map(|slot| Some(slot.retained_extra()));
        partition
            .entity_arena
            .select_retained_version(&context.execution, &id, selected, version)
            .map_err(|cause| selection_denial(record, cause))?;
        context
            .state
            .mark_entity_slot_touched_for_request(
                id.partition_id,
                id.slot_index(),
                &context.execution,
            )
            .map_err(request)?;
        Ok::<_, MutationApplyDenial>(before)
    })?;
    let after = workspace.with_context(|context| {
        context
            .state
            .retained_entity_extra(id.partition_id, id.slot_index())
    });
    outcome.record_change(
        &workspace.execution(),
        match before {
            Some(old_authoritative_aspect_state) => RecordMutation::EntityUpdated {
                entity_id: id,
                kind_id: kind,
                old_authoritative_aspect_state,
                new_authoritative_aspect_state: after,
                authoritative_patch: None,
            },
            None => RecordMutation::EntityRematerialized {
                entity_id: id,
                kind_id: kind,
                new_authoritative_aspect_state: after,
                authoritative_patch: None,
            },
        },
    )?;
    outcome.record_event(
        &workspace.execution(),
        MutationEvent::EntityUpdated { entity_id: id },
    )?;
    Ok(())
}
pub(super) fn retire(
    workspace: &mut MutationWorkspace<'_>,
    id: EntityId,
    version: VersionId,
    outcome: &mut MutationOutcome,
) -> Result<(), MutationApplyDenial> {
    let before = workspace.with_context(|context| {
        context
            .state
            .retained_entity_extra(id.partition_id, id.slot_index())
    });
    let kind = workspace.with_context(|context| {
        let partition = context
            .state
            .get_partition_mut_for_request(id.partition_id, &context.execution)
            .map_err(request)?;
        let kind = partition
            .entity_arena
            .get(&id)
            .filter(|slot| slot.is_live())
            .and_then(|slot| slot.kind_id())
            .ok_or_else(|| stale(RecordRef::Entity(id), "current owned entity unavailable"))?;
        partition
            .entity_arena
            .retire_for_request(&context.execution, id.slot_index(), version)
            .map_err(request)?
            .ok_or_else(|| stale(RecordRef::Entity(id), "current entity cannot retire"))?;
        context
            .state
            .mark_entity_slot_touched_for_request(
                id.partition_id,
                id.slot_index(),
                &context.execution,
            )
            .map_err(request)?;
        Ok::<_, MutationApplyDenial>(kind)
    })?;
    outcome.record_change(
        &workspace.execution(),
        RecordMutation::EntityDeleted {
            entity_id: id,
            kind_id: kind,
            old_authoritative_aspect_state: before,
            authoritative_patch: None,
        },
    )?;
    outcome.record_event(
        &workspace.execution(),
        MutationEvent::EntityDeleted { entity_id: id },
    )?;
    Ok(())
}
