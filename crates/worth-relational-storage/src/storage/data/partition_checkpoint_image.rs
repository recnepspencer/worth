//! Portable rows and adjacency belonging to one recorded partition.
use std::{collections::BTreeMap, marker::PhantomData};

use serde::{Deserialize, Serialize};
use worth_foundational::facade::PortableRecordAspectState;

use crate::identity::data::{
    EntityId, KindId, LineageId, PartitionId, RelationId, StructuralFingerprint, VersionId,
};
use crate::storage::data::{RecordLifecycleState, RelationalFieldRevision};
use crate::symbols::data::Symbol;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableBitSet {
    #[serde(default)]
    pub words: Vec<u64>,
    #[serde(default)]
    pub sparse_words: Vec<(u64, u64)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedEntityMetadataImage {
    pub effective_at: VersionId,
    pub retired_at: Option<VersionId>,
    pub generation: u32,
    pub kind_id: KindId,
    pub lineage_id: Option<LineageId>,
    pub authoritative_aspect_state: Option<PortableRecordAspectState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityExtraImage {
    pub structural_fingerprint: Option<StructuralFingerprint>,
    pub lineage_id: Option<LineageId>,
    pub authoritative_aspect_state: Option<PortableRecordAspectState>,
}

pub trait RecordArenaCheckpointKind: Clone + PartialEq + Eq {
    type MetaImage: Clone + PartialEq + Eq;
    type ExtraImage: Clone + PartialEq + Eq;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityCheckpointImageKind;

impl RecordArenaCheckpointKind for EntityCheckpointImageKind {
    type MetaImage = VersionedEntityMetadataImage;
    type ExtraImage = EntityExtraImage;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationCheckpointImageKind;

impl RecordArenaCheckpointKind for RelationCheckpointImageKind {
    type MetaImage = VersionedRelationMetadataImage;
    type ExtraImage = RelationExtraImage;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(
    serialize = "K::MetaImage: Serialize, K::ExtraImage: Serialize",
    deserialize = "K::MetaImage: Deserialize<'de>, K::ExtraImage: Deserialize<'de>"
))]
#[serde(deny_unknown_fields)]
pub struct RecordArenaCheckpointImage<K: RecordArenaCheckpointKind> {
    #[serde(default)]
    pub slots: Vec<u64>,
    pub generations: Vec<u32>,
    pub lifecycle: Vec<RecordLifecycleState>,
    pub kind_ids: Vec<Option<KindId>>,
    pub metadata_history: Vec<Vec<K::MetaImage>>,
    pub created_at: Vec<VersionId>,
    pub retired_at: Vec<Option<VersionId>>,
    pub aspect_versions: Vec<BTreeMap<Symbol, u64>>,
    #[serde(default)]
    pub field_revisions: Vec<Option<BTreeMap<(Symbol, Symbol), RelationalFieldRevision>>>,
    pub extra: Vec<K::ExtraImage>,
    pub diagnostics_enrichment: Vec<BTreeMap<Symbol, String>>,
    pub branch_pins: Vec<u32>,
    pub replay_pins: Vec<u32>,
    pub snapshot_pins: Vec<u32>,
    pub live_bitset: DurableBitSet,
    pub reclaimable_bitset: DurableBitSet,
    #[serde(default)]
    pub free_list: Vec<u64>,
    #[serde(skip)]
    pub marker: PhantomData<K>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationEndpointsImage {
    pub source: EntityId,
    pub target: EntityId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelationExtraImage {
    pub endpoints: Option<RelationEndpointsImage>,
    pub authoritative_aspect_state: Option<PortableRecordAspectState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedRelationMetadataImage {
    pub effective_at: VersionId,
    pub retired_at: Option<VersionId>,
    pub generation: u32,
    pub kind_id: KindId,
    pub endpoints: RelationEndpointsImage,
    pub authoritative_aspect_state: Option<PortableRecordAspectState>,
}

pub type EntityArenaCheckpointImage = RecordArenaCheckpointImage<EntityCheckpointImageKind>;
pub type RelationArenaCheckpointImage = RecordArenaCheckpointImage<RelationCheckpointImageKind>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PartitionCheckpointImage {
    pub partition_id: PartitionId,
    pub entity_arena: EntityArenaCheckpointImage,
    pub relation_arena: RelationArenaCheckpointImage,
    pub adjacency: Vec<DurableAdjacencyEntry>,
    pub reverse_adjacency: Vec<DurableAdjacencyEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableAdjacencyEntry {
    pub slot: u64,
    pub relations: Vec<RelationId>,
    pub structural_revisions: Vec<(KindId, VersionId)>,
}
