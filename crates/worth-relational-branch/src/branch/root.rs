use std::sync::Arc;
type OncePublicationBases =
    std::sync::OnceLock<std::sync::Weak<dyn crate::history::retention::PublicationBasisAccess>>;

use super::target::RelationalBranchRootDescriptor;
use crate::branch::root_regions::RelationalPersistentRegionSet;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::identity::data::PartitionId;
use crate::storage::overlay::{PartitionAccess, PartitionState};
use crate::storage::RelationalPublishedPartitionDelta;

#[path = "root_allocation.rs"]
pub mod allocation;
#[path = "root_authoritative_payload_tests.rs"]
pub mod authoritative_payload_tests;
#[path = "root_axes.rs"]
pub mod axes;
#[path = "root_candidate.rs"]
pub mod candidate;
#[path = "root_capture.rs"]
pub mod capture;
#[path = "root_capture_preparation.rs"]
pub mod capture_preparation;
#[path = "root_checkpoint_projection.rs"]
pub mod checkpoint_projection;
#[path = "root_commit_evidence.rs"]
pub mod commit_evidence;
#[path = "root_completeness.rs"]
pub mod completeness;
#[path = "root_empty.rs"]
pub mod empty;
pub use commit_evidence::{RelationalOriginalRootCommitEvidence, RelationalRootCommitEvidence};
pub mod branch_court_doors_for_test;
#[path = "root_content_tests.rs"]
pub mod content_tests;
#[path = "root_identity.rs"]
pub mod identity;
#[path = "root_output_publication.rs"]
pub mod output_publication;
#[path = "root_owner_allocation_ledger.rs"]
pub mod owner_allocation_ledger;
#[cfg(test)]
#[path = "root_physical_linkage_tests.rs"]
mod physical_linkage_tests;
#[path = "root_projection.rs"]
pub mod projection;
#[cfg(test)]
#[path = "root_projection_tests.rs"]
mod projection_tests;
#[path = "root_readmission.rs"]
pub mod readmission;
#[path = "root_region_access.rs"]
pub mod region_access;
#[path = "root_schema.rs"]
pub mod schema;
#[path = "root_schema_meaning.rs"]
pub mod schema_meaning;
#[cfg(test)]
#[path = "root_tests.rs"]
mod tests;
#[path = "root_visibility.rs"]
pub mod visibility;

pub(super) use super::root_region::RelationalRootRegion;
pub use super::root_region::RelationalRootRegionObservation;
pub(super) use allocation::{reserve_arc_memory, reserve_box_memory};
pub use allocation::{
    RelationalRootAllocationVisitor, RelationalRootAuthoritativeAllocationKind,
    RelationalRootAuthoritativeAllocationObservation,
};
pub use identity::RelationalBranchRootIdentityIssuer;
pub use schema::{
    retained_compilation_tree_bytes, RelationalBranchRootSchemaAuthority,
    RetainedRelationalAspectPlans, RetainedRelationalSchemaRegistry,
};
pub use visibility::RelationalBranchVisibilityCommitment;

pub use super::root_capture_denial::RelationalBranchRootCaptureDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RelationalBranchRootPublicationCost {
    pub touched_regions: u64,
    pub reused_regions: u64,
    /// Exact immutable radix nodes allocated by this publication.
    pub persistent_index_path_nodes: u64,
    pub content_values_hashed: u64,
    pub new_authoritative_bytes: u64,
    pub copied_truth_bytes: u64,
    pub copied_commit_envelopes: u64,
    pub new_schema_authorities: u64,
    pub reused_schema_authorities: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationalRootCorrectnessIndex {
    AuthoritativeFallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelationalBranchRootAxes {
    pub storage_version: u64,
    /// Inspection/completeness digest, never a currentness selector.
    pub storage_root: [u8; 32],
    pub schema_root: [u8; 32],
    pub correctness_index: RelationalRootCorrectnessIndex,
    pub visibility: RelationalBranchVisibilityCommitment,
}

#[derive(Debug, Clone)]
struct RelationalCommittedBranchRoot {
    commit_change: Option<Arc<crate::branch::maintenance::CommitChange>>,
    descriptor: RelationalBranchRootDescriptor,
    evidence: RelationalRootCommitEvidence,
    axes: RelationalBranchRootAxes,
    publication_cost: RelationalBranchRootPublicationCost,
}

/// Immutable root selected by one branch reference.
///
/// A later root replaces only regions named by the storage owner's sealed
/// publication delta. Untouched region `Arc`s are retained exactly; no hash,
/// debug representation, or collision-prone equality proxy can authorize
/// reuse.
#[derive(Debug)]
pub struct RelationalBranchRoot {
    pub publication_bases: OncePublicationBases,
    content: crate::content::ContentDirectory,
    id: u64,
    regions: Arc<RelationalPersistentRegionSet>,
    entity_slot_count: usize,
    relation_slot_count: usize,
    content_accumulator: [u8; 32],
    pub output_publication_positions: crate::history::data::RequiredOutputPublicationPositions,
    schema_authority: Arc<RelationalBranchRootSchemaAuthority>,
    pub execution_contracts:
        std::sync::OnceLock<Arc<dyn super::root_execution_payload::RootExecutionPayload>>,
    committed: Option<RelationalCommittedBranchRoot>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
    pub managed_source_image:
        crate::branch::projection_view::managed_source::ManagedSourceRootImages,
}

impl AsRef<RelationalBranchRoot> for RelationalBranchRoot {
    fn as_ref(&self) -> &Self {
        self
    }
}

#[path = "root_managed_fork.rs"]
pub mod managed_fork;

#[derive(Debug, Clone)]
pub struct RelationalBranchRootState {
    pub(super) root: Arc<RelationalBranchRoot>,
}

pub struct PreparedRelationalBranchRootCapture {
    root: Arc<RelationalBranchRoot>,
}

impl PreparedRelationalBranchRootCapture {
    pub fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }

    pub fn into_root(self) -> Arc<RelationalBranchRoot> {
        self.root
    }
}

impl RelationalBranchRootState {
    pub fn new(root: Arc<RelationalBranchRoot>) -> Self {
        Self { root }
    }

    pub fn root(&self) -> &Arc<RelationalBranchRoot> {
        &self.root
    }
}

impl RelationalBranchRoot {
    pub fn prepare_capture(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        issuer: &RelationalBranchRootIdentityIssuer,
        partitions: &impl PartitionAccess,
        published_delta: &RelationalPublishedPartitionDelta,
        previous: Option<&Arc<Self>>,
        envelope: Arc<CanonicalCommitEnvelope>,
        content: &crate::content::publication::ContentPublication,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        symbols: &crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable,
        planned_schema: Option<&Arc<crate::branch::root::RelationalBranchRootSchemaAuthority>>,
    ) -> Result<PreparedRelationalBranchRootCapture, RelationalBranchRootCaptureDenial> {
        capture_preparation::prepare(
            execution,
            issuer,
            partitions,
            published_delta,
            previous,
            envelope,
            content,
            registry,
            symbols,
            planned_schema,
        )
    }
    pub fn retained_content(&self) -> &crate::content::ContentDirectory {
        &self.content
    }
    pub(super) fn regions_for_projection(&self) -> &RelationalPersistentRegionSet {
        &self.regions
    }

    pub fn project_snapshot(&self) -> projection::SnapshotProjectionRoot<'_> {
        use self::projection::{SnapshotProjectionRoot, SnapshotProjectionSchema};

        let (version, semantics, evidence) = self.query_schema_context();
        SnapshotProjectionRoot {
            regions: &self.regions,
            id: self.id,
            storage_version: self
                .axes()
                .map(|axes| crate::identity::data::VersionId(axes.storage_version)),
            schema: SnapshotProjectionSchema {
                registry: self.schema_authority.registry(),
                plans: self.schema_authority.aspect_plans(),
                digest: self.schema_authority.authority_digest(),
                version,
                semantics,
                evidence,
            },
        }
    }

    pub const fn id(&self) -> u64 {
        self.id
    }

    pub const fn entity_slot_count(&self) -> usize {
        self.entity_slot_count
    }

    pub const fn relation_slot_count(&self) -> usize {
        self.relation_slot_count
    }

    pub fn descriptor(&self) -> Option<&RelationalBranchRootDescriptor> {
        self.committed.as_ref().map(|root| &root.descriptor)
    }

    pub fn axes(&self) -> Option<RelationalBranchRootAxes> {
        self.committed.as_ref().map(|root| root.axes)
    }

    pub fn canonical_payload(&self) -> Option<Arc<RelationalOriginalRootCommitEvidence>> {
        self.commit_evidence()?.original_payload()
    }

    pub fn commit_id(&self) -> Option<CommitId> {
        self.committed
            .as_ref()
            .map(|root| root.evidence.receipt().commit_id)
    }

    pub fn canonical_envelope(&self) -> Option<&Arc<CanonicalCommitEnvelope>> {
        self.commit_evidence()
            .and_then(RelationalRootCommitEvidence::original)
    }

    pub fn commit_evidence(&self) -> Option<&RelationalRootCommitEvidence> {
        self.committed.as_ref().map(|root| &root.evidence)
    }

    pub fn commit_receipt(&self) -> Option<&crate::history::data::RelationalCommitReceipt> {
        self.commit_evidence()
            .map(RelationalRootCommitEvidence::receipt)
    }

    pub fn schema_authority(&self) -> &RelationalBranchRootSchemaAuthority {
        &self.schema_authority
    }

    pub fn retained_schema_authority(&self) -> Arc<RelationalBranchRootSchemaAuthority> {
        Arc::clone(&self.schema_authority)
    }

    pub fn storage_regions(
        &self,
    ) -> impl Iterator<
        Item = Result<
            RelationalRootRegionObservation,
            crate::storage::substrate::StorageAllocationOverflow,
        >,
    > + '_ {
        self.regions.values().map(|region| region.observation())
    }

    pub fn region_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.regions.values().map(|region| region.id)
    }

    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    pub fn has_materialization_unavailable(&self) -> bool {
        self.regions.materialization_unavailable_records() > 0
    }

    /// Logical authoritative partition payload bytes reachable from this root.
    /// Checked: refuses an unrepresentable bound or total with a typed overflow.
    pub fn logical_partition_payload_bytes(
        &self,
    ) -> Result<u64, crate::storage::substrate::StorageAllocationOverflow> {
        self.regions.values().try_fold(0_u64, |total, region| {
            total
                .checked_add(region.allocation_inventory()?.authoritative_bytes)
                .ok_or(crate::storage::substrate::StorageAllocationOverflow)
        })
    }

    pub fn publication_cost(&self) -> RelationalBranchRootPublicationCost {
        self.committed
            .as_ref()
            .map(|root| root.publication_cost)
            .unwrap_or_default()
    }

    pub fn reclaimable_unique_authoritative_bytes(
        &self,
    ) -> Result<u64, crate::storage::substrate::StorageAllocationOverflow> {
        use crate::storage::substrate::StorageAllocationOverflow;
        let mut bytes = std::mem::size_of::<Self>() as u64;
        bytes = bytes
            .checked_add(
                RelationalPersistentRegionSet::reclaimable_unique_authoritative_bytes(
                    &self.regions,
                )?,
            )
            .ok_or(StorageAllocationOverflow)?;
        if Arc::strong_count(&self.schema_authority) == 1 {
            bytes = bytes
                .checked_add(self.schema_authority.authoritative_allocation_bytes())
                .ok_or(StorageAllocationOverflow)?;
        }
        Ok(bytes)
    }

    pub fn links_envelope(&self, envelope: &Arc<CanonicalCommitEnvelope>) -> bool {
        self.committed.as_ref().is_some_and(|root| {
            root.evidence
                .original()
                .is_some_and(|original| Arc::ptr_eq(original, envelope))
                && root.axes.storage_root == *root.descriptor.truth_root()
                && root.axes.schema_root == *root.descriptor.schema_root()
            // Exact immutable envelope custody carries root admission's
            // visibility proof. Cold readmission and is_complete verify
            // it independently; artifact linkage does not reconstruct it.
        })
    }

    pub fn shares_canonical_envelope_with(&self, source: &Self) -> bool {
        match (&self.committed, &source.committed) {
            (Some(target), Some(source)) => target.evidence.shares_owner(&source.evidence),
            (None, None) => true,
            _ => false,
        }
    }
}

impl RelationalBranchRoot {
    #[doc(hidden)]
    pub fn regions_for_test(&self) -> &RelationalPersistentRegionSet {
        &self.regions
    }

    #[doc(hidden)]
    pub fn replace_regions_for_test(&mut self, regions: Arc<RelationalPersistentRegionSet>) {
        self.regions = regions;
    }
}
