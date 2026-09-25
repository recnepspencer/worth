//! Exact-basis single-entity projection borrows the selected root's values.

use crate::identity::data::EntityId;
use crate::storage::overlay::PartitionAccess;
use crate::visibility::snapshot_states::SnapshotStateBasis;

use super::{EntityProjectionRecord, ProjectionAspectScope, VisibilityProjectionView};

impl VisibilityProjectionView<'_> {
    pub(super) fn project_entity_record<T>(
        &self,
        entity: EntityId,
        scope: &ProjectionAspectScope,
        mut project: impl FnMut(EntityProjectionRecord<'_>) -> Option<T>,
    ) -> Option<T> {
        let SnapshotStateBasis::Exact(basis) = &self.basis else {
            // Historical reconstruction retains its own version-selection rules.
            let record = self.authoritative_entity_record(entity)?;
            return project(EntityProjectionRecord::new(&record, scope));
        };
        let root = basis.root();
        let partition = root.get_partition(entity.partition_id)?;
        let slot = partition.entity_arena.get_slot(entity.slot_index())?;
        if !slot.is_live()
            || (!entity.generation.is_zero() && slot.generation() != entity.generation_value())
        {
            return None;
        }
        let kind = root
            .schema_authority()
            .registry()
            .resolve_entity(slot.kind_id()?)
            .ok()?;
        let actual = EntityId::new(
            entity.partition_id,
            entity.local_slot_value(),
            slot.generation(),
        );
        let created = partition
            .entity_arena
            .created_at_for_slot(entity.slot_index())
            .expect("visible entity slot has creation metadata");
        project(EntityProjectionRecord::from_slot(
            actual, &slot, &kind, created, scope,
        ))
    }
}

#[cfg(test)]
mod tests;
