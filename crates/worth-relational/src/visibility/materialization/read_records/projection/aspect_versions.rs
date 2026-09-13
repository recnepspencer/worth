use worth_foundational::facade::AspectKey;

use crate::identity::data::EntityId;
use crate::storage::overlay::PartitionAccess;

use super::VisibilityProjectionView;

impl VisibilityProjectionView<'_> {
    /// Native revision for one aspect at this view's selected immutable basis.
    ///
    /// The outer option rejects a stale entity identity. The inner option is
    /// absent when that live entity has no installed value for the aspect.
    pub fn entity_aspect_version(
        &self,
        entity: EntityId,
        aspect: &AspectKey,
    ) -> Option<Option<u64>> {
        let root = self.basis.root()?;
        let partition = root.get_partition(entity.partition_id)?;
        let slot = partition.entity_arena.get_slot(entity.slot_index())?;
        if slot.generation() != entity.generation_value()
            || slot.partition_id() != entity.partition_id
        {
            return None;
        }
        let symbol = self
            .runtime
            .services
            .symbols
            .with_read(|symbols| symbols.symbol(aspect.as_str()));
        let versions = partition
            .entity_arena
            .aspect_versions_at(entity.slot_index())?;
        Some(symbol.and_then(|symbol| versions.get(&symbol).copied()))
    }
}
