use std::fmt::Debug;

use crate::identity::data::{
    EntityDomain, KindId, PartitionId, RecordId, RelationDomain, VersionId,
};
use crate::storage::data::{EntityReadRecord, RelationReadRecord};
use crate::storage::overlay::PartitionState;

use super::{
    EntityExtra, RecordArena, RelationExtra, VersionedEntityMetadata, VersionedRelationMetadata,
};

pub(crate) trait RecordKind: Clone + Debug + 'static {
    type Domain: Copy + Ord + Debug + 'static;
    type Meta: Clone + Debug + HistoricalMetadata;
    type Extra: Clone + Debug;
    type ReadRecord: Clone + Debug;

    fn arena(partition: &PartitionState) -> &RecordArena<Self>;
    fn arena_mut(partition: &mut PartitionState) -> &mut RecordArena<Self>;
    fn empty_extra() -> Self::Extra;
    fn copy_extra_for_request(
        extra: &Self::Extra,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self::Extra, crate::execution::RelationalExecutionDenialCause>;
    #[cfg(test)]
    fn unavailable_extra(extra: &Self::Extra) -> Self::Extra;
    fn retire_metadata(metadata: &mut Self::Meta, version_id: VersionId);
    fn metadata_for_create(
        kind_id: KindId,
        generation: u32,
        version_id: VersionId,
        extra: crate::storage::substrate::StorageAllocation<Self::Extra>,
    ) -> Option<Self::Meta>;
    fn metadata_extra(
        metadata: &Self::Meta,
    ) -> &crate::storage::substrate::StorageAllocation<Self::Extra>;
    /// Checked declared upper bound of the backing reachable from this extra
    /// beyond its carrier layout. Each holder counts shared backing again, so
    /// this is never an exact physical inventory. `None` is unrepresentable.
    fn extra_reachable_bound_bytes(extra: &Self::Extra) -> Option<u64>;
}

pub(crate) trait HistoricalMetadata: Sized {
    fn effective_at(&self) -> VersionId;
    fn retired_at(&self) -> Option<VersionId>;
    fn generation(&self) -> u32;
    fn fund_for_request(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause>;
}

#[derive(Debug, Clone)]
pub(crate) struct EntityRecordKind;

impl RecordKind for EntityRecordKind {
    type Domain = EntityDomain;
    type Meta = VersionedEntityMetadata;
    type Extra = EntityExtra;
    type ReadRecord = EntityReadRecord;

    fn arena(partition: &PartitionState) -> &RecordArena<Self> {
        &partition.entity_arena
    }

    fn arena_mut(partition: &mut PartitionState) -> &mut RecordArena<Self> {
        &mut partition.entity_arena
    }

    fn empty_extra() -> Self::Extra {
        EntityExtra::default()
    }

    fn copy_extra_for_request(
        extra: &Self::Extra,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self::Extra, crate::execution::RelationalExecutionDenialCause> {
        extra.copy_for_request(request)
    }

    #[cfg(test)]
    fn unavailable_extra(extra: &Self::Extra) -> Self::Extra {
        EntityExtra::from_unbudgeted_parts(None, extra.lineage_id, None)
    }

    fn retire_metadata(metadata: &mut Self::Meta, version_id: VersionId) {
        metadata.retired_at = Some(version_id);
    }

    fn metadata_for_create(
        kind_id: KindId,
        generation: u32,
        version_id: VersionId,
        extra: crate::storage::substrate::StorageAllocation<Self::Extra>,
    ) -> Option<Self::Meta> {
        Some(VersionedEntityMetadata::from_ordinary(
            version_id,
            None,
            generation,
            kind_id,
            extra.lineage_id,
            extra,
        ))
    }

    fn metadata_extra(
        metadata: &Self::Meta,
    ) -> &crate::storage::substrate::StorageAllocation<Self::Extra> {
        &metadata.extra
    }

    fn extra_reachable_bound_bytes(extra: &Self::Extra) -> Option<u64> {
        extra.reachable_bound_bytes()
    }
}

impl HistoricalMetadata for VersionedEntityMetadata {
    fn fund_for_request(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        VersionedEntityMetadata::fund_for_request(self, request)
    }
    fn effective_at(&self) -> VersionId {
        self.effective_at
    }

    fn retired_at(&self) -> Option<VersionId> {
        self.retired_at
    }

    fn generation(&self) -> u32 {
        self.generation
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RelationRecordKind;

impl RecordKind for RelationRecordKind {
    type Domain = RelationDomain;
    type Meta = VersionedRelationMetadata;
    type Extra = RelationExtra;
    type ReadRecord = RelationReadRecord;

    fn arena(partition: &PartitionState) -> &RecordArena<Self> {
        &partition.relation_arena
    }

    fn arena_mut(partition: &mut PartitionState) -> &mut RecordArena<Self> {
        &mut partition.relation_arena
    }

    fn empty_extra() -> Self::Extra {
        RelationExtra::default()
    }

    fn copy_extra_for_request(
        extra: &Self::Extra,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self::Extra, crate::execution::RelationalExecutionDenialCause> {
        extra.copy_for_request(request)
    }

    #[cfg(test)]
    fn unavailable_extra(_extra: &Self::Extra) -> Self::Extra {
        RelationExtra::default()
    }

    fn retire_metadata(metadata: &mut Self::Meta, version_id: VersionId) {
        metadata.retired_at = Some(version_id);
    }

    fn metadata_for_create(
        kind_id: KindId,
        generation: u32,
        version_id: VersionId,
        extra: crate::storage::substrate::StorageAllocation<Self::Extra>,
    ) -> Option<Self::Meta> {
        Some(VersionedRelationMetadata::from_ordinary(
            version_id,
            None,
            generation,
            kind_id,
            extra.endpoints.clone()?,
            extra,
        ))
    }

    fn metadata_extra(
        metadata: &Self::Meta,
    ) -> &crate::storage::substrate::StorageAllocation<Self::Extra> {
        &metadata.extra
    }

    fn extra_reachable_bound_bytes(extra: &Self::Extra) -> Option<u64> {
        extra.reachable_bound_bytes()
    }
}

impl HistoricalMetadata for VersionedRelationMetadata {
    fn fund_for_request(
        self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        VersionedRelationMetadata::fund_for_request(self, request)
    }
    fn effective_at(&self) -> VersionId {
        self.effective_at
    }

    fn retired_at(&self) -> Option<VersionId> {
        self.retired_at
    }

    fn generation(&self) -> u32 {
        self.generation
    }
}

pub(crate) fn partition_of<K: RecordKind>(id: &RecordId<K::Domain>) -> PartitionId {
    id.partition_id
}

pub(crate) fn slot_of<K: RecordKind>(id: &RecordId<K::Domain>) -> usize {
    id.slot_index()
}

pub(crate) fn generation_of<K: RecordKind>(id: &RecordId<K::Domain>) -> u32 {
    id.generation_value()
}
