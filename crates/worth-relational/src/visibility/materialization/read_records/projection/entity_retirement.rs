//! Point metadata for one exact retired entity at an admitted immutable root.
use super::VisibilityProjectionView;
use crate::identity::data::{EntityId, KindId, VersionId};
use crate::storage::data::RecordLifecycleState;
use crate::storage::overlay::PartitionAccess;

/// Read evidence of deletion, without an entity payload or live entity authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationalEntityRetirement {
    entity: EntityId,
    kind: KindId,
    created_at: VersionId,
    deleted_at: VersionId,
}

impl RelationalEntityRetirement {
    pub const fn entity_id(self) -> EntityId {
        self.entity
    }
    pub const fn kind_id(self) -> KindId {
        self.kind
    }
    pub const fn created_at_version(self) -> VersionId {
        self.created_at
    }
    pub const fn deleted_at_version(self) -> VersionId {
        self.deleted_at
    }
}

impl VisibilityProjectionView<'_> {
    /// One exact identity lookup. Missing, live, reused, wildcard, or historical
    /// identities have no retirement evidence. No aspect state is materialized.
    pub fn entity_retirement(&self, entity: EntityId) -> Option<RelationalEntityRetirement> {
        if !self.is_exact_basis() || entity.generation.is_zero() {
            return None;
        }
        let root = self.basis.root()?;
        let partition = root.get_partition(entity.partition_id)?;
        let slot = partition.entity_arena.get_slot(entity.slot_index())?;
        if slot.generation() != entity.generation_value()
            || slot.lifecycle() != RecordLifecycleState::DeletedRetained
        {
            return None;
        }
        let deleted_at = slot.retired_at()?;
        if deleted_at > self.version_id() {
            return None;
        }
        Some(RelationalEntityRetirement {
            entity,
            kind: slot.kind_id()?,
            created_at: partition
                .entity_arena
                .created_at_for_slot(entity.slot_index())?,
            deleted_at,
        })
    }
}

#[cfg(test)]
mod tests;
