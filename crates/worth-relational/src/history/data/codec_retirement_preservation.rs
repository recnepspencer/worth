//! Verify that retirement preserves every protected current physical record.
use worth_foundational::facade::{
    AuthoritativeRecordAspectState, ContractValidatedAspectValueView,
};

use super::codec_retirement_closure::{
    charge, request, CodecRetirementClosureStop, OwnedCodecRetirementClosure,
};
use crate::branch::RelationalBranchRoot;
use crate::identity::data::{EntityId, RelationId};
use crate::runtime::RecoveredCheckpointProgramActivationUpdate;

pub(crate) fn validate_retired_codec_closure(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    before: &RelationalBranchRoot,
    after: &RelationalBranchRoot,
    closure: &OwnedCodecRetirementClosure,
    activation: &RecoveredCheckpointProgramActivationUpdate,
) -> Result<(), CodecRetirementClosureStop> {
    let result = execution
        .in_scope(|lease| {
            worth_execution::ExecutionWorkCeiling::new(execution.work_ceiling())
                .run_with_checkpoint(lease, None, |context| {
                    validate_metered(before, after, closure, activation, context)
                })
        })
        .map_err(request)?
        .map_err(request)?
        .0;
    execution.consult().map_err(request)?;
    result
}

fn validate_metered(
    before: &RelationalBranchRoot,
    after: &RelationalBranchRoot,
    closure: &OwnedCodecRetirementClosure,
    activation: &RecoveredCheckpointProgramActivationUpdate,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
) -> Result<(), CodecRetirementClosureStop> {
    let mut activation_seen = false;
    charge(
        context,
        before
            .entity_slot_count()
            .saturating_add(after.entity_slot_count())
            .saturating_add(before.relation_slot_count())
            .saturating_add(after.relation_slot_count())
            .saturating_add(before.region_count())
            .saturating_add(after.region_count()),
    )?;
    if before.partition_ids_iter().count() != after.partition_ids_iter().count() {
        return Err("codec retirement changed protected partition roster".into());
    }
    for partition_id in before.partition_ids_iter() {
        charge(context, 1)?;
        let old = before
            .partition_state(partition_id)
            .ok_or("codec retirement introduced a partition")?;
        let new = after
            .partition_state(partition_id)
            .ok_or("codec retirement removed a partition")?;
        if old.entity_arena.occupied_slots_iter().count()
            != new.entity_arena.occupied_slots_iter().count()
        {
            return Err("codec retirement changed protected entity roster".into());
        }
        for slot in old.entity_arena.occupied_slots_iter() {
            charge(context, 1_usize.saturating_add(closure.entities.len()))?;
            let old = old
                .entity_arena
                .get_slot(slot)
                .ok_or("codec retirement created a protected entity")?;
            let new = new
                .entity_arena
                .get_slot(slot)
                .ok_or("codec retirement erased retained entity history")?;
            let id = EntityId::new(partition_id, slot as u64, old.generation());
            if old.generation() != new.generation() || old.kind_id() != new.kind_id() {
                return Err("codec retirement changed protected native identity".into());
            }
            for state in [
                old.extra().authoritative_aspect_state.as_ref(),
                new.extra().authoritative_aspect_state.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                for (_, value) in state.aspects().entries() {
                    charge(context, value.semantic_byte_width())?;
                }
            }
            if closure.entities.contains(&id) {
                if !old.is_live()
                    || new.lifecycle()
                        != crate::storage::data::RecordLifecycleState::DeletedRetained
                {
                    return Err("codec retirement did not delete its exact proven output".into());
                }
                if old.extra().lineage_id != new.extra().lineage_id
                    || old.extra().structural_fingerprint != new.extra().structural_fingerprint
                    || old.extra().authoritative_aspect_state
                        != new.extra().authoritative_aspect_state
                {
                    return Err(
                        "codec retirement changed retained historical output payload or lineage"
                            .into(),
                    );
                }
                continue;
            }
            if old.lifecycle() != new.lifecycle()
                || old.extra().lineage_id != new.extra().lineage_id
                || old.extra().structural_fingerprint != new.extra().structural_fingerprint
            {
                return Err("codec retirement changed a protected entity".into());
            }
            if id == activation.entity_id {
                compare_activation_payload(
                    old.extra().authoritative_aspect_state.as_ref(),
                    new.extra().authoritative_aspect_state.as_ref(),
                    activation,
                )?;
                activation_seen = true;
            } else if old.extra().authoritative_aspect_state
                != new.extra().authoritative_aspect_state
            {
                return Err("codec retirement changed protected payload".into());
            }
        }
        if old.relation_arena.occupied_slots_iter().count()
            != new.relation_arena.occupied_slots_iter().count()
        {
            return Err("codec retirement changed protected relation roster".into());
        }
        for slot in old.relation_arena.occupied_slots_iter() {
            charge(context, 1_usize.saturating_add(closure.relations.len()))?;
            let old = old
                .relation_arena
                .get_slot(slot)
                .ok_or("codec retirement introduced a relation")?;
            let new = new
                .relation_arena
                .get_slot(slot)
                .ok_or("codec retirement erased a relation")?;
            for state in [
                old.extra().authoritative_aspect_state.as_ref(),
                new.extra().authoritative_aspect_state.as_ref(),
            ]
            .into_iter()
            .flatten()
            {
                for (_, value) in state.aspects().entries() {
                    charge(context, value.semantic_byte_width())?;
                }
            }
            let id = RelationId::new(partition_id, slot as u64, old.generation());
            let retired = closure.relations.contains(&id);
            if retired
                && (!old.is_live()
                    || new.lifecycle()
                        != crate::storage::data::RecordLifecycleState::DeletedRetained)
            {
                return Err("codec retirement did not delete its exact proven relation".into());
            }
            if old.generation() != new.generation()
                || old.kind_id() != new.kind_id()
                || (!retired && old.lifecycle() != new.lifecycle())
                || old.extra().endpoints != new.extra().endpoints
                || old.extra().authoritative_aspect_state != new.extra().authoritative_aspect_state
            {
                return Err(
                    "codec retirement changed protected relation payload or identity".into(),
                );
            }
        }
    }
    if !activation_seen {
        return Err("authenticated program activation target is absent".into());
    }
    Ok(())
}

fn compare_activation_payload(
    old: Option<&AuthoritativeRecordAspectState>,
    new: Option<&AuthoritativeRecordAspectState>,
    activation: &RecoveredCheckpointProgramActivationUpdate,
) -> Result<(), String> {
    let old = old.ok_or("program activation has no predecessor payload")?;
    let new = new.ok_or("program activation has no successor payload")?;
    if old.aspects().len() != new.aspects().len() {
        return Err("program activation changed protected aspect roster".into());
    }
    let path = activation.field_locator.field_path().fields();
    if path.len() != 1 {
        return Err("program activation requires one native physical field".into());
    }
    let activation_key = activation.field_locator.aspect().aspect_key();
    let mut seen = false;
    for ((key, previous), (next_key, next)) in old.aspects().entries().zip(new.aspects().entries())
    {
        if key != next_key {
            return Err("program activation changed protected aspect roster".into());
        }
        if key != activation_key {
            if previous != next {
                return Err("program activation changed a sibling protected aspect".into());
            }
            continue;
        }
        if previous.contract() != next.contract() {
            return Err("program activation changed its protected field contract".into());
        }
        let (
            ContractValidatedAspectValueView::Struct(previous),
            ContractValidatedAspectValueView::Struct(next),
        ) = (previous.view(), next.view())
        else {
            return Err("program activation requires its installed struct field carrier".into());
        };
        if previous.len() != next.len() {
            return Err("program activation patch differs from its authenticated exact predecessor/successor".into());
        }
        let mut field_seen = false;
        for ((field, previous), (next_field, next)) in previous.fields().zip(next.fields()) {
            if field != next_field {
                return Err("program activation changed protected field roster".into());
            }
            if field == &path[0] {
                if previous != &activation.expected || next != &activation.replacement {
                    return Err("program activation patch differs from its authenticated exact predecessor/successor".into());
                }
                field_seen = true;
            } else if previous != next {
                return Err("program activation changed a sibling protected field".into());
            }
        }
        if !field_seen {
            return Err("program activation field is absent".into());
        }
        seen = true;
    }
    if !seen {
        return Err("program activation aspect is absent".into());
    }
    Ok(())
}
