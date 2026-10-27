use worth_foundational::facade::AspectKey;

use crate::identity::data::EntityId;

use super::{RelationalBorrowedRecordReadDenial, VisibilityProjectionView};

impl VisibilityProjectionView {
    /// Native revision for one aspect at this view's selected exact immutable basis.
    ///
    /// The outer option rejects a stale entity identity. The inner option is
    /// absent when that live entity has no installed value for the aspect.
    pub fn read_entity_aspect_version(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        aspect: &AspectKey,
    ) -> Result<Option<Option<u64>>, RelationalBorrowedRecordReadDenial> {
        if !self.is_exact_basis() {
            return Err(RelationalBorrowedRecordReadDenial::ExactBasisRequired);
        }
        self.demand_regions_head(execution, [entity.partition_id])
            .map_err(RelationalBorrowedRecordReadDenial::Execution)?;
        Ok(self.selected_entity_aspect_version(entity, aspect))
    }
    fn selected_entity_aspect_version(
        &self,
        entity: EntityId,
        aspect: &AspectKey,
    ) -> Option<Option<u64>> {
        let partition = self.admitted_head_region(entity.partition_id)?;
        let slot = partition.entity_arena().get_slot(entity.slot_index())?;
        if slot.generation() != entity.generation_value()
            || slot.partition_id() != entity.partition_id
        {
            return None;
        }
        let symbol = self
            .symbols
            .with_read(|symbols| symbols.symbol(aspect.as_str()));
        let versions = partition
            .entity_arena()
            .aspect_versions_at(entity.slot_index())?;
        Some(symbol.and_then(|symbol| versions.get(&symbol).copied()))
    }
}
