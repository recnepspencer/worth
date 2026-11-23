//! Replace one field image in an unpublished partition copy for an SDK court.
use super::PartitionState;
use crate::identity::data::EntityId;
use worth_foundational::facade::{
    admit_authoritative_record_aspect_state, AspectFieldLocator, AspectValue,
};
use worth_proof::{Artifact, TransitionOutcome};

impl PartitionState {
    pub(crate) fn corrupt_entity_field_for_test(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        locator: &AspectFieldLocator,
        replacement: Option<AspectValue>,
    ) -> bool {
        use crate::storage::substrate::{EntityExtra, StorageAllocation};
        use worth_foundational::facade::AuthoritativeRecordAspectPatch;
        let [field] = locator.field_path().fields() else {
            return false;
        };
        let arena = &mut self.entity_arena;
        if arena.get(&entity).is_none() {
            return false;
        }
        let Some(slot) = arena.slots.physical_index(entity.local_slot.0 as usize) else {
            return false;
        };
        // Prepare independent values before replacing any original carrier. On
        // failure, the caller discards this unpublished clone of the pinned root.
        let prepared = (|| {
            let current = &arena.extra[slot];
            let state = current.authoritative_aspect_state()?;
            let last = arena.metadata_history[slot].len().checked_sub(1)?;
            let original_metadata = &arena.metadata_history[slot][last];
            let bounds = AuthoritativeRecordAspectPatch::empty()
                .application_retained_storage_bounds(Some(state))?;
            let replacement_bytes = replacement
                .as_ref()
                .map(AspectValue::clone_allocation_bytes)
                .unwrap_or(Some(0))?;
            let scratch_bytes = bounds
                .retained_bytes
                .checked_add(bounds.transient_bytes)?
                .checked_add(replacement_bytes.checked_mul(2)?)
                .and_then(|bytes| u64::try_from(bytes).ok())?;
            // The existing bound owns the independent state construction;
            // replacement argument and rebuilt field coexist during corruption.
            let scratch = execution.reserve_retained_memory(scratch_bytes).ok()?;
            let state = execution
                .run_native_with_checkpoint(|context| {
                    crate::execution::request_stage::checkpoint(context, scratch_bytes).ok()?;
                    let mut changed = false;
                    let entries = state.aspects().entries().map(|(key, value)| {
                        let mut value = value.clone();
                        if key == locator.aspect().aspect_key() {
                            changed =
                                value.corrupt_struct_field_for_test(field, replacement.clone());
                        }
                        Artifact::new(value)
                    });
                    let TransitionOutcome::Success(state) =
                        admit_authoritative_record_aspect_state(entries)
                    else {
                        return None;
                    };
                    changed.then(|| state.into_parts().into_parts().0)
                })
                .ok()?
                .0?;
            let prepare_extra = |prior: &EntityExtra| {
                let bounds = AuthoritativeRecordAspectPatch::empty()
                    .application_retained_storage_bounds(Some(&state))?;
                let bytes = u64::try_from(bounds.retained_bytes)
                    .ok()?
                    .checked_add(StorageAllocation::<EntityExtra>::layout_bytes())?;
                let memory = execution.reserve_retained_memory(bytes).ok()?;
                Some(StorageAllocation::new(EntityExtra::from_funded_parts(
                    prior.structural_fingerprint,
                    prior.lineage_id,
                    Some(state.clone()),
                    memory,
                )))
            };
            let extra = prepare_extra(current)?;
            let mut metadata = original_metadata.clone();
            // Preserve every version/identity field and the original metadata
            // Extra's other meaning; only the requested field image changes.
            metadata.extra = prepare_extra(&original_metadata.extra)?;
            let metadata = metadata.fund_for_request(execution).ok()?;
            Some((last, extra, metadata, scratch))
        })();
        let Some((last, extra, metadata, _scratch)) = prepared else {
            return false;
        };
        let mut history = arena.metadata_history[slot].clone();
        if history.set_for_request(last, metadata, execution).is_err()
            || arena
                .metadata_history
                .set_for_request(slot, history, execution)
                .is_err()
        {
            return false;
        }
        let Ok(Some(target)) = arena.extra.carrier_mut_for_request(slot, execution) else {
            return false;
        };
        *target = extra;
        true
    }
}
