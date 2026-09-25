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
    type Meta: Clone + Debug;
    type Extra: Clone + Debug;
    type ReadRecord: Clone + Debug;

    fn arena(partition: &PartitionState) -> &RecordArena<Self>;
    fn arena_mut(partition: &mut PartitionState) -> &mut RecordArena<Self>;
    fn empty_extra() -> Self::Extra;
    fn unavailable_extra(extra: &Self::Extra) -> Self::Extra;
    fn retire_metadata(metadata: &mut Self::Meta, version_id: VersionId);
    fn metadata_for_create(
        kind_id: KindId,
        generation: u32,
        version_id: VersionId,
        extra: &Self::Extra,
    ) -> Self::Meta;
    fn metadata_owned_allocation_bytes(metadata: &Self::Meta) -> u64;
    fn extra_owned_allocation_bytes(extra: &Self::Extra) -> u64;
}

pub(crate) trait HistoricalMetadata {
    fn effective_at(&self) -> VersionId;
    fn retired_at(&self) -> Option<VersionId>;
    fn generation(&self) -> u32;
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

    fn unavailable_extra(extra: &Self::Extra) -> Self::Extra {
        EntityExtra {
            structural_fingerprint: None,
            lineage_id: extra.lineage_id,
            authoritative_aspect_state: None,
        }
    }

    fn retire_metadata(metadata: &mut Self::Meta, version_id: VersionId) {
        metadata.retired_at = Some(version_id);
    }

    fn metadata_for_create(
        kind_id: KindId,
        generation: u32,
        version_id: VersionId,
        extra: &Self::Extra,
    ) -> Self::Meta {
        VersionedEntityMetadata {
            effective_at: version_id,
            retired_at: None,
            generation,
            kind_id,
            lineage_id: extra.lineage_id,
            authoritative_aspect_state: extra.authoritative_aspect_state.clone(),
        }
    }

    fn metadata_owned_allocation_bytes(metadata: &Self::Meta) -> u64 {
        metadata
            .authoritative_aspect_state
            .as_ref()
            .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64)
    }

    fn extra_owned_allocation_bytes(extra: &Self::Extra) -> u64 {
        extra
            .authoritative_aspect_state
            .as_ref()
            .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64)
    }
}

impl HistoricalMetadata for VersionedEntityMetadata {
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
        extra: &Self::Extra,
    ) -> Self::Meta {
        VersionedRelationMetadata {
            effective_at: version_id,
            retired_at: None,
            generation,
            kind_id,
            endpoints: extra
                .endpoints
                .clone()
                .expect("relation metadata requires endpoints"),
            authoritative_aspect_state: extra.authoritative_aspect_state.clone(),
        }
    }

    fn metadata_owned_allocation_bytes(metadata: &Self::Meta) -> u64 {
        metadata
            .authoritative_aspect_state
            .as_ref()
            .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64)
    }

    fn extra_owned_allocation_bytes(extra: &Self::Extra) -> u64 {
        extra
            .authoritative_aspect_state
            .as_ref()
            .map_or(0, |state| state.owned_allocation_capacity_bytes() as u64)
    }
}

impl HistoricalMetadata for VersionedRelationMetadata {
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
