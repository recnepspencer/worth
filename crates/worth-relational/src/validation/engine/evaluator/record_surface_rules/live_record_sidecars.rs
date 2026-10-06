use crate::diagnostics::data::DiagnosticCode;
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::{EntityRecordKind, RecordKind, RelationRecordKind, SlotView};
use crate::validation::data::{
    InvariantClass, InvariantViolation, InvariantViolationFields, RecordKindTag,
    StorageInconsistencyScan,
};

use super::super::super::context::InvariantExecutionContext;
use super::super::common::{storage_inconsistency_violation, StorageInconsistencyContext};

pub(super) fn evaluate_live_record_sidecar_rule(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    kind: &RecordKindTag,
) -> Option<InvariantViolation> {
    match kind {
        RecordKindTag::Entity => evaluate_live_record_sidecar::<EntityRecordKind>(
            context,
            class,
            |state, partition_id| state.touched_entity_slots(partition_id),
            |state, partition_id| state.has_touched_entity_slots(partition_id),
            |slot_view| slot_view.kind_id().is_some(),
            "kind id",
            |context, slots| context.metrics().count_entity_slot_scans(slots),
        ),
        RecordKindTag::Relation => evaluate_live_record_sidecar::<RelationRecordKind>(
            context,
            class,
            |state, partition_id| state.touched_relation_slots(partition_id),
            |state, partition_id| state.has_touched_relation_slots(partition_id),
            |slot_view| slot_view.extra().endpoints.is_some(),
            "endpoints",
            |context, slots| context.metrics().count_relation_slot_scans(slots),
        ),
    }
}

fn evaluate_live_record_sidecar<K: RecordKind>(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    touched_slots: impl Fn(
        &dyn crate::runtime::PartitionAccess,
        crate::identity::data::PartitionId,
    ) -> Option<Vec<usize>>,
    has_touched_slots: impl Fn(
        &dyn crate::runtime::PartitionAccess,
        crate::identity::data::PartitionId,
    ) -> bool,
    has_required_sidecar: impl Fn(&SlotView<'_, K>) -> bool,
    missing_label: &str,
    count_scans: impl Fn(&InvariantExecutionContext<'_, '_>, usize),
) -> Option<InvariantViolation> {
    let mut has_touched_surface = false;
    for partition_id in context.partition_access().partition_ids_iter() {
        if !context.checkpoint(1) {
            return None;
        }
        if has_touched_slots(context.partition_access(), partition_id) {
            has_touched_surface = true;
            break;
        }
    }

    for partition_id in context.partition_access().partition_ids_iter() {
        if !context.checkpoint(1) {
            return None;
        }
        let Some(partition) = context.partition_access().get_partition(partition_id) else {
            if !context.claim_result(4096) {
                return None;
            }
            return Some(storage_inconsistency_violation(
                class,
                format!(
                    "partition {:?} missing during invariant sidecar scan",
                    partition_id
                ),
                StorageInconsistencyContext::default()
                    .with_partition_id(partition_id)
                    .with_scan(StorageInconsistencyScan::LiveRecordSidecar),
            ));
        };
        let has_touched_partition = has_touched_slots(context.partition_access(), partition_id);
        let touched_slots_for_partition = if has_touched_partition {
            let arena = K::arena(partition);
            if !context.check_scratch_peak(
                (arena.slot_count() as u64).saturating_mul(std::mem::size_of::<usize>() as u64),
            ) {
                return None;
            }
            touched_slots(context.partition_access(), partition_id)
        } else {
            None
        };
        if let Some(slots) = touched_slots_for_partition {
            count_scans(context, slots.len());
            for slot in slots {
                if !context.checkpoint(1) {
                    return None;
                }
                if !context.check_result_peak(4096) {
                    return None;
                }
                if let Some(violation) = sidecar_violation_for_slot(
                    class,
                    partition,
                    slot,
                    &has_required_sidecar,
                    missing_label,
                ) {
                    if !context.claim_result(4096) {
                        return None;
                    }
                    return Some(violation);
                }
            }
        } else if !has_touched_surface {
            let arena = K::arena(partition);
            count_scans(context, arena.slot_count());
            for slot in arena.occupied_slots_iter() {
                if !context.checkpoint(1) {
                    return None;
                }
                if !context.check_result_peak(4096) {
                    return None;
                }
                if let Some(violation) = sidecar_violation_for_slot(
                    class,
                    partition,
                    slot,
                    &has_required_sidecar,
                    missing_label,
                ) {
                    if !context.claim_result(4096) {
                        return None;
                    }
                    return Some(violation);
                }
            }
        }
    }
    None
}

fn sidecar_violation_for_slot<K: RecordKind>(
    class: InvariantClass,
    partition: &crate::storage::overlay::PartitionState,
    slot: usize,
    has_required_sidecar: &impl Fn(&SlotView<'_, K>) -> bool,
    missing_label: &str,
) -> Option<InvariantViolation> {
    let slot_view = K::arena(partition).get_slot(slot)?;
    if slot_view.lifecycle() == RecordLifecycleState::Live && !has_required_sidecar(&slot_view) {
        return Some(InvariantViolation {
            class,
            code: DiagnosticCode::SidecarConsistencyFailure,
            detail: format!(
                "live slot {} in partition {} missing {}",
                slot,
                partition.partition_id.as_u32(),
                missing_label
            ),
            fields: InvariantViolationFields::SidecarConsistency {
                partition_id: partition.partition_id,
                slot,
                missing_label: missing_label.to_string(),
            },
        });
    }
    None
}
