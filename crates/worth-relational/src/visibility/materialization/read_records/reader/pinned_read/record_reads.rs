//! Single-record materialization from the selected immutable basis.
use super::*;
use crate::identity::data::{EntityId, RelationId, VersionId};
use crate::storage::{
    data::{EntityReadRecord, RelationReadRecord},
    overlay::PartitionAccess,
};
impl RelationalPinnedRead {
    pub(crate) fn authoritative_entity_record_for_id_at_version_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &RelationalSchemaRegistry,
        id: EntityId,
        version: VersionId,
    ) -> Option<EntityReadRecord> {
        super::super::record_identity_materialization::authoritative_entity_record_for_id_at_version_with_registry(state, registry, id, version, self.current_version)
    }
    pub(crate) fn authoritative_relation_record_for_id_at_version_with_registry(
        &self,
        state: &(impl PartitionAccess + ?Sized),
        registry: &RelationalSchemaRegistry,
        id: RelationId,
        version: VersionId,
    ) -> Option<RelationReadRecord> {
        super::super::record_identity_materialization::authoritative_relation_record_for_id_at_version_with_registry(state, registry, id, version, self.current_version)
    }
}
