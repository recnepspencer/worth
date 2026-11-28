use std::collections::BTreeMap;
use worth_relational_contracts::internal_port::durability::artifact_payload_emission::digest::Sha256Sink;

use serde::{Deserialize, Serialize};
use worth_foundational::facade::PortableAspectContract;

use super::CheckpointCoverage;
use worth_relational_branch::internal_port::history::data::positioned_canonical_commit::PositionedCanonicalCommit;
use worth_relational_branch::internal_port::indexes::data::DerivedIndexArtifacts;
use worth_relational_contracts::internal_port::branch::reference_checkpoint_data::RelationalBranchCellCheckpoint;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;
use worth_relational_contracts::internal_port::indexes::data::definition::DerivedIndexDefinition;
use worth_relational_contracts::internal_port::lineage::data::artifacts::checkpoint_artifact::LineageCheckpointArtifact;
use worth_relational_contracts::internal_port::symbols::data::symbol_table_snapshot::SymbolTableSnapshot;
use worth_relational_storage::internal_port::storage::data::record_identity_image::{
    DurableRecordGenerationHighWater, DurableRecordIdentityState, DurableRecordSlotFrontier,
    DurableReusableRecordSlot,
};

use worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::PartitionCheckpointImage;

/// Versioned checkpoint payload for one exact immutable branch root.
///
/// Commit identity selects the root artifact. Branch cells independently
/// select that commit, allowing every branch that shares a root to reuse one
/// recovered owner artifact without duplicating authoritative partitions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableBranchRootImage {
    #[serde(default)]
    pub content_records:
        worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
    #[serde(default)]
    pub format_version: u16,
    pub commit_id: CommitId,
    pub partition_images: Vec<PartitionCheckpointImage>,
    /// Integrity of the exact reconstructive partition images. The committed
    /// branch target remains the canonical root identity; this digest prevents
    /// recovery from accepting different reconstructive bytes under it.
    pub partition_image_digest: [u8; 32],
    #[serde(default)]
    pub schema_carrier_digest: [u8; 32],
    #[serde(default)]
    pub root_image_digest: [u8; 32],
}

impl DurableBranchRootImage {
    // Version 6 binds the complete Original v4 payload projection together with
    // both parents' current schema, index and field-revision provenance.
    // Earlier descriptors are refused before readmission, never reinterpreted.
    pub const CURRENT_FORMAT_VERSION: u16 = 6;
}

pub fn branch_root_partition_image_digest(
    _lane: &worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    partition_images: &[PartitionCheckpointImage],
) -> Result<[u8; 32], rmp_serde::encode::Error> {
    use sha2::Digest;

    let mut digest = Sha256Sink::default();
    digest.0.update(b"worth.relational.branch-root-images.v1\0");
    digest
        .0
        .update((partition_images.len() as u64).to_be_bytes());
    rmp_serde::encode::write(&mut digest, partition_images)?;
    Ok(digest.0.finalize().into())
}

pub fn branch_root_image_digest(
    _lane: &worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
    format_version: u16,
    commit_id: CommitId,
    partition_image_digest: [u8; 32],
    schema_carrier_digest: [u8; 32],
    content_records: &worth_relational_storage::internal_port::content::attachments::RelationalContentAttachments,
) -> [u8; 32] {
    use sha2::{Digest, Sha256};

    let mut digest = Sha256::new();
    digest.update(b"worth.relational.branch-root-image.v1\0");
    digest.update(format_version.to_be_bytes());
    digest.update(commit_id.0.to_be_bytes());
    digest.update(partition_image_digest);
    digest.update(schema_carrier_digest);
    if !content_records.is_empty() {
        digest.update(b"worth.native.content-root-closure.v1\0");
        for record in content_records.records() {
            digest.update(record.identity().0);
        }
    }
    digest.finalize().into()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DurableCheckpoint {
    pub(crate) recorded_components:
        BTreeMap<CommitId, Vec<worth_relational_branch::internal_port::branch::basis_descriptor::RelationalBranchBasisDescriptor>>,
    pub(crate) retained_empty_components: Vec<super::RelationalCheckpointEmptyComponent>,
    pub(crate) root_kind_images:
        BTreeMap<CommitId, worth_relational_storage::internal_port::storage::residency::checkpoint_kind_images::CheckpointRegionKindImages>,
    pub(crate) mirror_kind_images: worth_relational_storage::internal_port::storage::residency::checkpoint_kind_images::CheckpointRegionKindImages,
    pub coverage: CheckpointCoverage,
    /// Deferred selected-medium restoration owns its original cold directory.
    pub(crate) directory: Option<super::NativeCheckpointDirectoryEdge>,
    /// Eager/Embedded restoration retains ORIGINAL canonical bytes as DATA.
    /// Empty materialized images are valid and cannot select the cold path.
    pub(crate) canonical_directory: Option<super::NativeCheckpointDirectoryEdge>,
    pub(crate) commit_descriptors:
        BTreeMap<CommitId, worth_relational_contracts::internal_port::branch::target::RelationalBranchRootDescriptor>,
    pub(crate) mirror_schema_digest: Option<[u8; 32]>,
    pub(crate) branch_cells: Vec<RelationalBranchCellCheckpoint>,
    /// One posture for every branch cell, in branch order.
    pub(crate) branch_lifecycle: Vec<worth_relational_branch::internal_port::durability::data::branch_lifecycle_images::DurableBranchLifecycleImage>,
    /// Every retired branch name, sorted; none may be reused.
    pub(crate) retired_branch_names: Vec<worth_relational_contracts::internal_port::history::data::coordinates::BranchId>,
    /// Immutable root images selected by the checkpoint's branch cells.
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
    pub(crate) index_generation_frontier: u64,
    pub(crate) index_definition_frontier: u64,
    pub index_definitions: Vec<DerivedIndexDefinition>,
    pub derived_index_artifacts: DerivedIndexArtifacts,
    pub(crate) derived_index_checkpoint:
        Option<crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>,
    pub(crate) derived_index_checkpoint_format: u16,
    pub symbol_table: SymbolTableSnapshot,
    pub(crate) symbol_identity_frontier: u32,
    pub runtime_name: String,
}

#[doc(hidden)]
pub trait DurableCheckpointOwnerPort: Sized {
    fn directory_for_retirement(
        &mut self,
        canonical: bool,
    ) -> Option<&mut super::NativeCheckpointDirectoryEdge>;

    fn envelopes(&self) -> &[PositionedCanonicalCommit];
    fn selected_directory(&self) -> Option<&super::NativeCheckpointDirectoryEdge>;
    fn original_canonical_directory(&self) -> Option<&super::NativeCheckpointDirectoryEdge>;
    fn derived_index_checkpoint(
        &self,
    ) -> Option<&crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts>;
}

impl DurableCheckpointOwnerPort for DurableCheckpoint {
    fn directory_for_retirement(
        &mut self,
        canonical: bool,
    ) -> Option<&mut super::NativeCheckpointDirectoryEdge> {
        if canonical {
            self.canonical_directory.as_mut()
        } else {
            self.directory.as_mut()
        }
    }

    fn envelopes(&self) -> &[PositionedCanonicalCommit] {
        &self.envelopes
    }
    fn selected_directory(&self) -> Option<&super::NativeCheckpointDirectoryEdge> {
        self.directory.as_ref()
    }
    fn original_canonical_directory(&self) -> Option<&super::NativeCheckpointDirectoryEdge> {
        self.canonical_directory.as_ref()
    }
    fn derived_index_checkpoint(
        &self,
    ) -> Option<&crate::durability::derived_index_artifacts::DerivedIndexCheckpointArtifacts> {
        self.derived_index_checkpoint.as_ref()
    }
}

mod fixture_port;
pub use fixture_port::DurableCheckpointFieldsFixturePort;

#[cfg(test)]
pub mod tests {
    use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::RecordArenaFixturePort as _;

    use sha2::{Digest, Sha256};

    use super::branch_root_partition_image_digest;
    use crate::durability::checkpoints::{
        aspect_state_images::CheckpointAspectContractCatalog,
        images::partition_to_image_with_contracts,
    };

    #[test]
    fn streamed_branch_root_image_digest_matches_buffered_wire_for_divergent_roots() {
        worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test(|fixture_lane| {
            let lane = fixture_lane;
            let fixture = crate::durability::native_region_fixture::populated_partition();
            let contracts = CheckpointAspectContractCatalog::from_contracts(
                fixture.schema.retained_aspect_contracts(),
            )
            .unwrap();
            let mut sibling = fixture.partition.clone();
            let kind = worth_relational_contracts::internal_port::identity::data::primitives::KindId(1);
            sibling
                .entity_arena
                .push_slot_for_test(worth_relational_storage::internal_port::storage::substrate::record_arena::arena::SlotInit {
                    partition_id: fixture.partition.partition_id,
                    kind_id: kind,
                    version_id: worth_relational_contracts::internal_port::identity::data::primitives::VersionId(13),
                    extra: Default::default(),
                });
            let roots = [fixture.partition, sibling].map(|partition| {
                vec![partition_to_image_with_contracts(
                    lane,
                    partition,
                    fixture.schema.aspect_plans(),
                    &contracts,
                )
                .unwrap()]
            });
            assert_eq!(roots.len(), 2);
            assert_ne!(roots[0], roots[1]);
            for root in &roots {
                assert_eq!(
                    branch_root_partition_image_digest(lane, root).unwrap(),
                    buffered_digest(root)
                );
            }
            let mut altered = roots[0].clone();
            let original = branch_root_partition_image_digest(lane, &altered).unwrap();
            altered[0].entity_arena.generations[0] ^= 1;
            assert_eq!(
                branch_root_partition_image_digest(lane, &altered).unwrap(),
                buffered_digest(&altered)
            );
            assert_ne!(
                branch_root_partition_image_digest(lane, &altered).unwrap(),
                original
            );
        })
    }

    fn buffered_digest(
        images: &[worth_relational_storage::internal_port::storage::data::partition_checkpoint_image::PartitionCheckpointImage],
    ) -> [u8; 32] {
        let encoded = rmp_serde::to_vec(images).unwrap();
        let mut digest = Sha256::new();
        digest.update(b"worth.relational.branch-root-images.v1\0");
        digest.update((images.len() as u64).to_be_bytes());
        digest.update(encoded);
        digest.finalize().into()
    }
}
