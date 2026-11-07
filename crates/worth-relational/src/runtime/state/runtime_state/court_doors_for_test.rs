//! Corruption targets the actual pinned storage image selected for an SDK read.
use super::RelationalRuntime;
use crate::{identity::data::EntityId, snapshots::data::SnapshotHandle};
use worth_foundational::facade::{
    admit_authoritative_record_aspect_state, AspectFieldLocator, AspectValue,
};
use worth_proof::{Artifact, TransitionOutcome};

impl RelationalRuntime {
    #[doc(hidden)]
    pub fn corrupt_snapshot_field_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        snapshot: &SnapshotHandle,
        entity: EntityId,
        locator: &AspectFieldLocator,
        replacement: Option<AspectValue>,
    ) -> bool {
        if snapshot.runtime_instance_id() != self.runtime_instance_id() {
            return false;
        }
        let Some(binding) = self
            .visibility
            .handles
            .active_binding(snapshot.snapshot_id())
        else {
            return false;
        };
        let root = self.services.symbols.with_read(|symbols| {
            crate::storage::substrate::maintenance::ReconstructionLane::enter_reconstruction(
                |lane| {
                    binding.basis.root().corrupt_field_image_for_test(
                        lane,
                        execution,
                        entity,
                        locator,
                        replacement,
                        symbols,
                    )
                },
            )
        });
        let Some(root) = root else { return false };
        let basis = crate::visibility::snapshot_states::VisibilitySnapshotBasis::from_applied_root(
            binding.branch_id,
            binding.version_id,
            root,
        );
        // Retain the issued snapshot identity and its original closed read policy.
        self.visibility
            .handles
            .remove_active(snapshot.snapshot_id());
        self.visibility.handles.insert_active(
            snapshot.snapshot_id(),
            crate::runtime::state::subsystems::SnapshotHandleBinding::new(
                basis,
                binding.read_policy,
            ),
        );
        self.visibility.cache.clear();
        true
    }
}

impl crate::storage::overlay::PartitionState {
    pub(crate) fn corrupt_entity_field_for_test(
        &mut self,
        entity: EntityId,
        locator: &AspectFieldLocator,
        replacement: Option<AspectValue>,
    ) -> bool {
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
        let Some(state) = arena.extra[slot].authoritative_aspect_state.as_ref() else {
            return false;
        };
        let mut changed = false;
        let entries = state.aspects().entries().map(|(key, value)| {
            let mut value = value.clone();
            if key == locator.aspect().aspect_key() {
                changed = value.corrupt_struct_field_for_test(field, replacement.clone());
            }
            Artifact::new(value)
        });
        let TransitionOutcome::Success(state) = admit_authoritative_record_aspect_state(entries)
        else {
            return false;
        };
        if !changed {
            return false;
        }
        let (state, _, _) = state.into_parts().into_parts();
        // Current reads and reads at the current exact version share this damaged image.
        let Some(metadata) = arena.metadata_history[slot].last_mut() else {
            return false;
        };
        metadata.authoritative_aspect_state = Some(state.clone());
        arena.extra[slot].authoritative_aspect_state = Some(state);
        true
    }
}
