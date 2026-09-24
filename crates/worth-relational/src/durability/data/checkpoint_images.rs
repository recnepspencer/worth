use std::collections::BTreeMap;
use std::marker::PhantomData;

use serde::{Deserialize, Serialize};
use worth_foundational::facade::{PortableAspectContract, PortableRecordAspectState};

use super::CheckpointCoverage;
use crate::branch::RelationalBranchCellCheckpoint;
use crate::history::data::CommitId;
use crate::history::data::PositionedCanonicalCommit;
use crate::identity::data::{
    EntityId, KindId, LineageId, PartitionId, RelationId, StructuralFingerprint, VersionId,
};
use crate::indexes::data::{DerivedIndexArtifacts, DerivedIndexDefinition};
use crate::lineage::data::LineageCheckpointArtifact;
use crate::storage::data::{RecordLifecycleState, RelationalFieldRevision};
use crate::symbols::data::{Symbol, SymbolTableSnapshot};

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

/// Versioned checkpoint payload for one exact immutable branch root.
///
/// Commit identity selects the root artifact. Branch cells independently
/// select that commit, allowing every branch that shares a root to reuse one
/// recovered owner artifact without duplicating authoritative partitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableBranchRootImage {
    #[serde(default)]
    pub(crate) format_version: u16,
    pub commit_id: CommitId,
    pub partition_images: Vec<PartitionCheckpointImage>,
    /// Integrity of the exact reconstructive partition images. The committed
    /// branch target remains the canonical root identity; this digest prevents
    /// recovery from accepting different reconstructive bytes under it.
    pub partition_image_digest: [u8; 32],
    #[serde(default)]
    pub(crate) schema_carrier_digest: [u8; 32],
    #[serde(default)]
    pub(crate) root_image_digest: [u8; 32],
}

impl DurableBranchRootImage {
    // Version 4 binds native field-revision provenance into each arena image.
    // Earlier descriptors cannot be readmitted under the new commitment grammar.
    pub(crate) const CURRENT_FORMAT_VERSION: u16 = 4;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DurableRecordGenerationClass {
    Entity,
    Relation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurableRecordGenerationHighWater {
    pub(crate) class: DurableRecordGenerationClass,
    pub(crate) partition_id: PartitionId,
    pub(crate) slot: u64,
    pub(crate) generation: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurableReusableRecordSlot {
    pub(crate) class: DurableRecordGenerationClass,
    pub(crate) partition_id: PartitionId,
    pub(crate) slot: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurableRecordSlotFrontier {
    pub(crate) class: DurableRecordGenerationClass,
    pub(crate) partition_id: PartitionId,
    pub(crate) next_slot: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DurableRecordReservationOrigin {
    AppendFrontier,
    Reclaimed { prior_generation: u32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurablePendingRecordReservation {
    pub(crate) class: DurableRecordGenerationClass,
    pub(crate) partition_id: PartitionId,
    pub(crate) slot: u64,
    pub(crate) generation: u32,
    pub(crate) origin: DurableRecordReservationOrigin,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct DurableRecordIdentityState {
    pub(crate) schema_version: u16,
    #[serde(default)]
    pub(crate) generation_high_water: Vec<DurableRecordGenerationHighWater>,
    #[serde(default)]
    pub(crate) reusable_slots: Vec<DurableReusableRecordSlot>,
    #[serde(default)]
    pub(crate) append_frontiers: Vec<DurableRecordSlotFrontier>,
    #[serde(default)]
    pub(crate) pending_reservations: Vec<DurablePendingRecordReservation>,
}

impl DurableRecordIdentityState {
    pub(crate) const CURRENT_SCHEMA_VERSION: u16 = 2;

    pub(crate) fn current(
        generation_high_water: Vec<DurableRecordGenerationHighWater>,
        reusable_slots: Vec<DurableReusableRecordSlot>,
        append_frontiers: Vec<DurableRecordSlotFrontier>,
        pending_reservations: Vec<DurablePendingRecordReservation>,
    ) -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA_VERSION,
            generation_high_water,
            reusable_slots,
            append_frontiers,
            pending_reservations,
        }
    }
}

pub(crate) fn branch_root_partition_image_digest(
    partition_images: &[PartitionCheckpointImage],
) -> Result<[u8; 32], rmp_serde::encode::Error> {
    use sha2::{Digest, Sha256};

    let mut digest = Sha256::new();
    digest.update(b"worth.relational.branch-root-images.v1\0");
    digest.update((partition_images.len() as u64).to_be_bytes());
    rmp_serde::encode::write(&mut Sha256Writer(&mut digest), partition_images)?;
    Ok(digest.finalize().into())
}

struct Sha256Writer<'a>(&'a mut sha2::Sha256);

impl std::io::Write for Sha256Writer<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        use sha2::Digest;

        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub(crate) fn branch_root_image_digest(
    format_version: u16,
    commit_id: CommitId,
    partition_image_digest: [u8; 32],
    schema_carrier_digest: [u8; 32],
) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut digest = Sha256::new();
    digest.update(b"worth.relational.branch-root-image.v1\0");
    digest.update(format_version.to_be_bytes());
    digest.update(commit_id.0.to_be_bytes());
    digest.update(partition_image_digest);
    digest.update(schema_carrier_digest);
    digest.finalize().into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableCheckpoint {
    pub coverage: CheckpointCoverage,
    pub(crate) branch_cells: Vec<RelationalBranchCellCheckpoint>,
    pub(crate) branch_roots: Vec<DurableBranchRootImage>,
    pub(crate) branch_root_schema_images: Vec<super::DurableBranchRootSchemaImage>,
    pub(crate) record_identity: DurableRecordIdentityState,
    /// Version-zero migration fields retained only for decoding old images.
    pub(crate) record_generation_high_water: Vec<DurableRecordGenerationHighWater>,
    pub(crate) reusable_record_slots: Vec<DurableReusableRecordSlot>,
    pub(crate) record_slot_frontiers: Vec<DurableRecordSlotFrontier>,
    pub(crate) envelopes: Vec<PositionedCanonicalCommit>,
    pub partition_images: Vec<PartitionCheckpointImage>,
    pub aspect_contracts: Vec<PortableAspectContract>,
    pub lineage: LineageCheckpointArtifact,
    pub index_definitions: Vec<DerivedIndexDefinition>,
    pub derived_index_artifacts: DerivedIndexArtifacts,
    pub(crate) derived_index_checkpoint:
        Option<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>,
    pub(crate) derived_index_checkpoint_format: u16,
    pub symbol_table: SymbolTableSnapshot,
    pub runtime_name: String,
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::branch_root_partition_image_digest;
    use crate::tests::support::*;

    #[test]
    fn streamed_branch_root_image_digest_matches_buffered_wire_for_divergent_roots() {
        let runtime = persisted_runtime_with_test_schema();
        create_entity(&runtime, "streamed-image-seed");
        let sibling = create_branch_from_main(&runtime, "streamed-image-sibling");
        let changed = create_entity_outcome_on_branch(&runtime, "streamed-image-fork", sibling);
        release_test_commit_snapshot(&runtime, &changed);
        let checkpoint = runtime.durability_authority().checkpoint().unwrap();
        assert_eq!(checkpoint.branch_roots.len(), 2);

        for root in &checkpoint.branch_roots {
            assert_eq!(
                branch_root_partition_image_digest(&root.partition_images).unwrap(),
                buffered_digest(&root.partition_images)
            );
        }
        let mut altered = checkpoint.branch_roots[0].partition_images.clone();
        let original = branch_root_partition_image_digest(&altered).unwrap();
        altered[0].entity_arena.generations[0] ^= 1;
        assert_eq!(
            branch_root_partition_image_digest(&altered).unwrap(),
            buffered_digest(&altered)
        );
        assert_ne!(
            branch_root_partition_image_digest(&altered).unwrap(),
            original
        );
    }

    fn buffered_digest(images: &[super::PartitionCheckpointImage]) -> [u8; 32] {
        let encoded = rmp_serde::to_vec(images).unwrap();
        let mut digest = Sha256::new();
        digest.update(b"worth.relational.branch-root-images.v1\0");
        digest.update((images.len() as u64).to_be_bytes());
        digest.update(encoded);
        digest.finalize().into()
    }
}
