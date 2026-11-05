//! Relation slot transitions retain selected full-GEN endpoints and payloads.
use super::*;
pub(super) fn select(
    workspace: &mut MutationWorkspace<'_>,
    source: &crate::branch::RelationalBranchRoot,
    id: RelationId,
    version: VersionId,
    outcome: &mut MutationOutcome,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), MutationApplyDenial> {
    let record = RecordRef::Relation(id);
    let arena = source
        .get_current_partition(id.partition_id)
        .ok_or_else(|| stale(record.clone(), "selected relation partition unavailable"))?
        .relation_arena();
    let selected = arena
        .get(&id)
        .filter(|slot| slot.is_live())
        .ok_or_else(|| stale(record.clone(), "selected full-GEN relation unavailable"))?;
    let kind = selected
        .kind_id()
        .ok_or_else(|| stale(record.clone(), "selected relation kind unavailable"))?;
    let endpoints = selected
        .extra()
        .endpoints
        .as_ref()
        .ok_or_else(|| stale(record.clone(), "selected endpoints unavailable"))?
        .clone();
    let payload = selected.extra().authoritative_aspect_state.clone();
    crate::execution::request_stage::checkpoint(
        context,
        payload
            .as_ref()
            .map_or(0, |p| p.owned_allocation_capacity_bytes()) as u64,
    )
    .map_err(request)?;
    let before = workspace.with_context(|context| {
        let partition = context.state.get_partition_mut(id.partition_id);
        let before = partition
            .relation_arena
            .get(&id)
            .filter(|slot| slot.is_live())
            .map(|slot| {
                (
                    slot.extra().endpoints.clone(),
                    slot.extra().authoritative_aspect_state.clone(),
                )
            });
        partition
            .relation_arena
            .select_retained_version(&id, selected, version)
            .map_err(|detail| stale(record.clone(), detail))?;
        context
            .state
            .mark_relation_slot_touched(id.partition_id, id.slot_index());
        Ok::<_, CommitConflict>(before)
    })?;
    outcome.record_change(match before {
        Some((old_endpoints, old_authoritative_aspect_state)) => {
            let old = old_endpoints
                .ok_or_else(|| stale(record, "current relation endpoints unavailable"))?;
            RecordMutation::RelationUpdated {
                relation_id: id,
                kind_id: kind,
                old_source: old.source,
                old_target: old.target,
                new_source: endpoints.source,
                new_target: endpoints.target,
                old_authoritative_aspect_state,
                new_authoritative_aspect_state: payload,
                authoritative_patch: None,
            }
        }
        None => RecordMutation::RelationRematerialized {
            relation_id: id,
            kind_id: kind,
            source: endpoints.source,
            target: endpoints.target,
            new_authoritative_aspect_state: payload,
            authoritative_patch: None,
        },
    });
    outcome.record_event(MutationEvent::RelationUpdated { relation_id: id });
    Ok(())
}
pub(super) fn retire(
    workspace: &mut MutationWorkspace<'_>,
    id: RelationId,
    version: VersionId,
    outcome: &mut MutationOutcome,
) -> Result<(), MutationApplyDenial> {
    let (kind, endpoints, payload) = workspace.with_context(|context| {
        let record = RecordRef::Relation(id);
        let partition = context.state.get_partition_mut(id.partition_id);
        let old = partition
            .relation_arena
            .get(&id)
            .filter(|slot| slot.is_live())
            .ok_or_else(|| stale(record.clone(), "current owned relation unavailable"))?;
        let kind = old
            .kind_id()
            .ok_or_else(|| stale(record.clone(), "current relation kind unavailable"))?;
        let endpoints = old
            .extra()
            .endpoints
            .clone()
            .ok_or_else(|| stale(record.clone(), "current relation endpoints unavailable"))?;
        let payload = old.extra().authoritative_aspect_state.clone();
        partition
            .relation_arena
            .retire(id.slot_index(), version)
            .ok_or_else(|| stale(record, "current relation cannot retire"))?;
        context
            .state
            .mark_relation_slot_touched(id.partition_id, id.slot_index());
        Ok::<_, CommitConflict>((kind, endpoints, payload))
    })?;
    outcome.record_change(RecordMutation::RelationDeleted {
        relation_id: id,
        kind_id: kind,
        source: endpoints.source,
        target: endpoints.target,
        authoritative_aspect_state: payload,
    });
    outcome.record_event(MutationEvent::RelationDeleted { relation_id: id });
    Ok(())
}
