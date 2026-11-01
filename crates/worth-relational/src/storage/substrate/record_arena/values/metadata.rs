//! Historical metadata retains the actual immutable current-extra allocation.
use super::{EntityExtra, RelationEndpoints, RelationExtra};
use crate::identity::data::{KindId, LineageId, VersionId};
use crate::storage::substrate::StorageAllocation;
use serde::ser::SerializeStruct;
use worth_foundational::facade::AuthoritativeRecordAspectState;
pub(crate) struct VersionedEntityMetadata {
    pub(crate) effective_at: VersionId,
    pub(crate) retired_at: Option<VersionId>,
    pub(crate) generation: u32,
    pub(crate) kind_id: KindId,
    pub(crate) lineage_id: Option<LineageId>,
    pub(crate) extra: StorageAllocation<EntityExtra>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(crate) struct VersionedRelationMetadata {
    pub(crate) effective_at: VersionId,
    pub(crate) retired_at: Option<VersionId>,
    pub(crate) generation: u32,
    pub(crate) kind_id: KindId,
    pub(crate) endpoints: RelationEndpoints,
    pub(crate) extra: StorageAllocation<RelationExtra>,
    _funding: Option<worth_execution::ExecutionMemoryReservation>,
}
impl VersionedEntityMetadata {
    pub(crate) fn authoritative_aspect_state(&self) -> Option<&AuthoritativeRecordAspectState> {
        self.extra.authoritative_aspect_state()
    }
}
impl VersionedRelationMetadata {
    pub(crate) fn authoritative_aspect_state(&self) -> Option<&AuthoritativeRecordAspectState> {
        self.extra.authoritative_aspect_state()
    }
}
impl serde::Serialize for VersionedEntityMetadata {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = serializer.serialize_struct("VersionedEntityMetadata", 6)?;
        value.serialize_field("effective_at", &self.effective_at)?;
        value.serialize_field("retired_at", &self.retired_at)?;
        value.serialize_field("generation", &self.generation)?;
        value.serialize_field("kind_id", &self.kind_id)?;
        value.serialize_field("lineage_id", &self.lineage_id)?;
        value.serialize_field(
            "authoritative_aspect_state",
            &self.authoritative_aspect_state(),
        )?;
        value.end()
    }
}
impl serde::Serialize for VersionedRelationMetadata {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut value = serializer.serialize_struct("VersionedRelationMetadata", 6)?;
        value.serialize_field("effective_at", &self.effective_at)?;
        value.serialize_field("retired_at", &self.retired_at)?;
        value.serialize_field("generation", &self.generation)?;
        value.serialize_field("kind_id", &self.kind_id)?;
        value.serialize_field("endpoints", &self.endpoints)?;
        value.serialize_field(
            "authoritative_aspect_state",
            &self.authoritative_aspect_state(),
        )?;
        value.end()
    }
}

mod ownership;
#[cfg(test)]
mod tests;
