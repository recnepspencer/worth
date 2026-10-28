use crate::diagnostics::data::DiagnosticCode;
use crate::validation::data::{
    InvariantClass, InvariantViolation, InvariantViolationFields, StorageInconsistencyScan,
};

use super::super::super::context::InvariantExecutionContext;
use super::super::common::{storage_inconsistency_violation, StorageInconsistencyContext};

pub(super) fn evaluate_max_snapshot_entities(
    context: &InvariantExecutionContext<'_, '_>,
    class: InvariantClass,
    limit: usize,
) -> Option<InvariantViolation> {
    let state_view = context.state_view();
    let mut visible_entities = 0;
    for partition_id in state_view.state().partition_ids_iter() {
        if !context.checkpoint(1) {
            return None;
        }
        let Some(partition) = state_view.state().get_current_partition(partition_id) else {
            return Some(storage_inconsistency_violation(
                class,
                format!(
                    "partition {:?} missing during snapshot entity count",
                    partition_id
                ),
                StorageInconsistencyContext::default()
                    .with_partition_id(partition_id)
                    .with_scan(if state_view.version_id() == context.current_version_id() {
                        StorageInconsistencyScan::MaxSnapshotEntities
                    } else {
                        StorageInconsistencyScan::HistoricalMaxSnapshotEntities
                    }),
            ));
        };
        if state_view.version_id() == context.current_version_id()
            && state_view
                .state()
                .base_current_partition(partition_id)
                .is_none()
        {
            if !context.checkpoint(partition.entity_arena().slot_count() as u64) {
                return None;
            }
            visible_entities += partition.entity_arena().live_count();
            continue;
        }
        let slots = state_view.try_entity_slots(partition_id, |work, bytes| {
            if context.checkpoint(work) && context.claim_scratch(bytes) {
                Ok(())
            } else {
                Err(())
            }
        });
        let Ok(slots) = slots else {
            return None;
        };
        for slot in slots {
            if !context.checkpoint(1) {
                return None;
            }
            context.metrics().count_entity_slot_scans(1);
            visible_entities += usize::from(
                state_view
                    .entity_metadata_for_slot(partition_id, slot)
                    .is_some(),
            );
        }
    }
    if visible_entities > limit {
        if !context.claim_result(4096) {
            return None;
        }
        return Some(InvariantViolation {
            class,
            code: DiagnosticCode::InvariantViolation,
            detail: format!(
                "snapshot at version {} has {} entities, limit is {}",
                state_view.version_id().as_u64(),
                visible_entities,
                limit
            ),
            fields: InvariantViolationFields::SnapshotEntityLimit {
                version_id: state_view.version_id(),
                visible_entities,
                limit,
            },
        });
    }
    None
}
